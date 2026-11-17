// Kernel tests: `weft_core_tests <bundle dir> <case>` runs one case against
// a fake platform shell. Bundles are the compiled files of tests/js/.

#include <weft/ops.hpp>
#include <weft/runtime.h>

#include <algorithm>
#include <chrono>
#include <cstdio>
#include <cstring>
#include <fstream>
#include <functional>
#include <iterator>
#include <map>
#include <optional>
#include <set>
#include <sstream>
#include <stdexcept>
#include <string>
#include <thread>
#include <vector>

namespace {

struct Failure : std::runtime_error {
  using std::runtime_error::runtime_error;
};

#define CHECK(cond)                                                                     \
  do {                                                                                  \
    if (!(cond)) throw Failure(std::string(__FILE__ ":") + std::to_string(__LINE__) + \
                               ": CHECK(" #cond ") failed");                            \
  } while (0)

#define CHECK_EQ(a, b)                                                                     \
  do {                                                                                     \
    auto &&va = (a);                                                                       \
    auto &&vb = (b);                                                                       \
    if (!(va == vb)) {                                                                     \
      std::ostringstream message;                                                          \
      message << __FILE__ ":" << __LINE__ << ": " #a " == " #b " failed:\n  " << va        \
              << "\n  " << vb;                                                             \
      throw Failure(message.str());                                                        \
    }                                                                                      \
  } while (0)

std::string dir;

std::vector<uint8_t> read_file(const std::string &path) {
  std::ifstream in(path, std::ios::binary);
  if (!in) throw Failure("cannot read " + path);
  return {std::istreambuf_iterator<char>(in), {}};
}

/// A platform shell that records everything the kernel sends.
struct Shell {
  struct Call {
    uint32_t id;
    std::string module, method;
    std::vector<uint8_t> args;
  };
  std::vector<std::pair<int, std::string>> logs;
  std::vector<std::vector<uint8_t>> commits;
  std::vector<Call> calls;
  std::optional<uint64_t> tick_in;
  std::set<std::string> modules{"echo"};
  std::vector<uint8_t> bytecode;
  weft_runtime *rt = nullptr;

  weft_host host() {
    weft_host h{};
    h.ctx = this;
    h.commit = [](void *ctx, const uint8_t *ops, size_t len) {
      static_cast<Shell *>(ctx)->commits.emplace_back(ops, ops + len);
    };
    h.schedule = [](void *ctx, uint64_t delay) {
      auto &shell = *static_cast<Shell *>(ctx);
      shell.tick_in = shell.tick_in ? std::min(*shell.tick_in, delay) : delay;
    };
    h.supports = [](void *ctx, const char *module) {
      return static_cast<Shell *>(ctx)->modules.count(module) > 0;
    };
    h.call = [](void *ctx, uint32_t id, const char *module, const char *method, const uint8_t *args,
                size_t len) {
      static_cast<Shell *>(ctx)->calls.push_back({id, module, method, {args, args + len}});
    };
    h.log = [](void *ctx, int level, const char *msg, size_t len) {
      static_cast<Shell *>(ctx)->logs.emplace_back(level, std::string(msg, len));
    };
    h.version = "17.0";
    return h;
  }

  weft_runtime *start(const std::string &bundle, uint64_t hash = 0) {
    bytecode = read_file(dir + "/" + bundle + ".hbc");
    weft_host h = host();
    rt = weft_runtime_new(&h, bytecode.data(), bytecode.size(), hash);
    return rt;
  }

  /// Ticks whenever asked to, sleeping until the requested time, until no timer is left.
  void pump() {
    for (int i = 0; i < 1000 && tick_in; i++) {
      std::this_thread::sleep_for(std::chrono::milliseconds(*tick_in));
      tick_in.reset();
      weft_runtime_tick(rt);
    }
    CHECK(!tick_in);
  }

  std::vector<std::string> lines(int level = WEFT_LOG_INFO) const {
    std::vector<std::string> out;
    for (auto &[l, text] : logs) {
      if (l == level) out.push_back(text);
    }
    return out;
  }

  bool logged(int level, const std::string &part) const {
    return std::any_of(logs.begin(), logs.end(), [&](auto &entry) {
      return entry.first == level && entry.second.find(part) != std::string::npos;
    });
  }

  ~Shell() { weft_runtime_free(rt); }
};

std::ostream &operator<<(std::ostream &out, const std::vector<std::string> &lines) {
  out << '[';
  for (size_t i = 0; i < lines.size(); i++) out << (i ? ", " : "") << '"' << lines[i] << '"';
  return out << ']';
}

size_t index_of(const std::vector<std::string> &lines, const std::string &line) {
  auto found = std::find(lines.begin(), lines.end(), line);
  if (found == lines.end()) throw Failure("missing log line: " + line);
  return size_t(found - lines.begin());
}

void timers() {
  Shell shell;
  CHECK(shell.start("timers"));
  CHECK(shell.tick_in.has_value());
  // Deliberately deliver the first tick late. OS timer granularity (notably
  // on Windows) cannot guarantee a tick between the 10 ms and 20 ms timers.
  // Overdue timers run in deadline order; nested timers and repeats must
  // still wait for the next tick, rather than running in this batch.
  std::this_thread::sleep_for(std::chrono::milliseconds(50));
  shell.tick_in.reset();
  weft_runtime_tick(shell.rt);
  CHECK_EQ(shell.lines(), (std::vector<std::string>{"a 0", "args 1 two", "interval 1", "outer", "b 20", "done"}));
  CHECK(shell.tick_in.has_value());
  shell.pump();
  auto lines = shell.lines();
  CHECK(index_of(lines, "done") < index_of(lines, "nested"));
  CHECK_EQ(std::count_if(lines.begin(), lines.end(), [](auto &l) { return l.rfind("interval", 0) == 0; }), 3);
  index_of(lines, "interval 3");
  CHECK(std::find(lines.begin(), lines.end(), "cancelled") == lines.end());
}

void microtasks() {
  Shell shell;
  CHECK(shell.start("microtasks"));
  CHECK_EQ(shell.lines(), (std::vector<std::string>{"sync", "promise", "micro", "after boom"}));
  CHECK(shell.logged(WEFT_LOG_ERROR, "boom"));
  shell.pump();
  CHECK_EQ(shell.lines(), (std::vector<std::string>{"sync", "promise", "micro", "after boom", "timer 1",
                                                    "micro of timer 1", "timer 2"}));
}

void console() {
  Shell shell;
  CHECK(shell.start("console"));
  auto &logs = shell.logs;
  CHECK_EQ(logs.size(), size_t(7));
  CHECK_EQ(logs[0].second, std::string(R"(text 1 1.5 true null undefined {"a":[1,"b"]})"));
  CHECK_EQ(logs[1].first, WEFT_LOG_DEBUG);
  CHECK_EQ(logs[2].first, WEFT_LOG_INFO);
  CHECK_EQ(logs[3].first, WEFT_LOG_WARN);
  CHECK_EQ(logs[4].first, WEFT_LOG_ERROR);
  CHECK_EQ(logs[4].second, std::string("e"));
  CHECK_EQ(logs[5].second, std::string("[object Object]"));
  CHECK_EQ(logs[6].second.rfind("Error: bad", 0), size_t(0));
}

void text() {
  Shell shell;
  CHECK(shell.start("text"));
  CHECK_EQ(shell.lines(), (std::vector<std::string>{"10 héllo ✓"}));
}

void calls() {
  Shell shell;
  CHECK(shell.start("calls"));
  CHECK_EQ(shell.lines(), (std::vector<std::string>{
                              "supports true false version 17.0",
                              R"(unsupported weft: platform module "nope" is not supported)",
                          }));
  CHECK_EQ(shell.calls.size(), size_t(3));
  auto &upper = shell.calls[0];
  CHECK_EQ(upper.module, std::string("echo"));
  CHECK_EQ(upper.method, std::string("upper"));
  CHECK(upper.args == (std::vector<uint8_t>{'h', 'i'}));
  CHECK(shell.calls[1].args == (std::vector<uint8_t>{2, 3}));
  CHECK(shell.calls[2].args.empty());

  const uint8_t hi[] = {'H', 'I'};
  weft_runtime_resolve(shell.rt, upper.id, true, hi, sizeof hi);
  const char nope[] = "nope!";
  weft_runtime_resolve(shell.rt, shell.calls[1].id, false, reinterpret_cast<const uint8_t *>(nope), 5);
  weft_runtime_resolve(shell.rt, shell.calls[2].id, true, nullptr, 0);
  // Completed and unknown ids are ignored.
  weft_runtime_resolve(shell.rt, upper.id, true, hi, sizeof hi);
  weft_runtime_resolve(shell.rt, 99, true, hi, sizeof hi);
  auto lines = shell.lines();
  CHECK_EQ(std::vector<std::string>(lines.begin() + 2, lines.end()),
           (std::vector<std::string>{"resolved HI", "rejected nope!", "empty 0"}));
}

/// A payload: one encoded value.
std::vector<uint8_t> payload(const std::string &text) {
  std::vector<uint8_t> out{3, uint8_t(text.size()), 0, 0, 0};
  out.insert(out.end(), text.begin(), text.end());
  return out;
}

void session() {
  Shell shell;
  CHECK(shell.start("session", 0xab));
  CHECK_EQ(shell.commits.size(), size_t(1));
  auto &batch = shell.commits[0];
  weft::ops::Reader reader(batch.data(), batch.size());
  weft::ops::Op op;
  CHECK(reader.next(op));
  CHECK_EQ(weft::ops::format(op), std::string("create 7 0"));
  CHECK(!reader.next(op));

  auto send = [&](const std::vector<uint8_t> &bytes) {
    return weft_runtime_dispatch(shell.rt, 3, 1, bytes.data(), bytes.size());
  };
  CHECK(send(payload("ok")));
  CHECK(!send(payload("reject")));
  CHECK(send({1}));
  std::vector<uint8_t> number{2, 0, 0, 0, 0, 0, 0, 4, 64};  // 2.5
  CHECK(send(number));
  CHECK(send({}));
  // A malformed payload is reported and ignored.
  CHECK(send({9}));
  CHECK(shell.logged(WEFT_LOG_ERROR, "unknown value tag 9"));
  CHECK(send(payload("close")));
  CHECK(send(payload("after close")));
  CHECK_EQ(shell.lines(), (std::vector<std::string>{
                              "dispatch 3 1 string ok",
                              "dispatch 3 1 string reject",
                              "dispatch 3 1 boolean true",
                              "dispatch 3 1 number 2.5",
                              "dispatch 3 1 undefined undefined",
                              "dispatch 3 1 string close",
                          }));
}

void refused() {
  {
    Shell shell;
    CHECK(!shell.start("session", 0xac));
    CHECK(shell.logged(WEFT_LOG_ERROR, "layout mismatch (native views 00000000000000ac, app 00000000000000ab)"));
  }
  {
    Shell shell;
    CHECK(!shell.start("throws"));
    CHECK(shell.logged(WEFT_LOG_ERROR, "bad bundle"));
  }
  {
    Shell shell;
    std::vector<uint8_t> junk(64, 7);
    weft_host h = shell.host();
    CHECK(!weft_runtime_new(&h, junk.data(), junk.size(), 0));
    CHECK(shell.logged(WEFT_LOG_ERROR, "the bundle failed to start"));
  }
}

void stable_rows() {
  auto stable = [](std::vector<uint32_t> old_rows, std::vector<uint32_t> new_rows) {
    std::vector<uint8_t> out(new_rows.size(), 9);
    size_t count = weft_stable_rows(old_rows.data(), old_rows.size(), new_rows.data(), new_rows.size(), out.data());
    CHECK_EQ(count, size_t(std::count(out.begin(), out.end(), 1)));
    return out;
  };
  CHECK(stable({1, 2, 3, 4, 5}, {5, 1, 2, 6, 3}) == (std::vector<uint8_t>{0, 1, 1, 0, 1}));
  CHECK(stable({1, 2, 3}, {3, 2, 1}) == (std::vector<uint8_t>{0, 0, 1}));
  CHECK(stable({1, 2, 3}, {1, 2, 3}) == (std::vector<uint8_t>{1, 1, 1}));
  CHECK(stable({}, {4, 5}) == (std::vector<uint8_t>{0, 0}));
  CHECK(stable({1, 2}, {}).empty());
}

/// Encodes ops for the decoder tests.
struct Batch {
  std::vector<uint8_t> bytes{weft::ops::kVersion};
  Batch &u8(uint8_t v) {
    bytes.push_back(v);
    return *this;
  }
  Batch &u32(uint32_t v) {
    for (int i = 0; i < 4; i++) bytes.push_back(uint8_t(v >> (8 * i)));
    return *this;
  }
  Batch &number(double v) {
    uint64_t bits;
    std::memcpy(&bits, &v, 8);
    u8(2);
    for (int i = 0; i < 8; i++) bytes.push_back(uint8_t(bits >> (8 * i)));
    return *this;
  }
  Batch &string(const std::string &s) {
    u8(3).u32(uint32_t(s.size()));
    bytes.insert(bytes.end(), s.begin(), s.end());
    return *this;
  }
};

std::vector<std::string> decode(const std::vector<uint8_t> &bytes) {
  weft::ops::Reader reader(bytes.data(), bytes.size());
  std::vector<std::string> out;
  for (weft::ops::Op op; reader.next(op);) out.push_back(weft::ops::format(op));
  return out;
}

bool rejects(const std::vector<uint8_t> &bytes) {
  try {
    decode(bytes);
    return false;
  } catch (const weft::ops::Error &) {
    return true;
  }
}

void ops() {
  Batch b;
  b.u8(1).u32(1).u32(0);
  b.u8(2).u32(1).u32(0).string("a\"\\\n\x01é");
  b.u8(2).u32(1).u32(1).u8(1);
  for (double v : {0.1, 123.0, -0.0, 1.5e-7, 1e21, 123456789012345680000.0, -2.5}) b.u8(2).u32(1).u32(2).number(v);
  b.u8(4).u32(2).u32(0).u32(2).u32(3).u32(4);
  b.u8(4).u32(2).u32(0).u32(0);
  b.u8(3).u32(1).u32(0).u32(0);
  b.u8(5).u32(0).u32(0).u32(1);
  CHECK_EQ(decode(b.bytes), (std::vector<std::string>{
                                "create 1 0",
                                R"(set 1 0 "a\"\\\n\u0001é")",
                                "set 1 1 true",
                                "set 1 2 0.1",
                                "set 1 2 123",
                                "set 1 2 0",
                                "set 1 2 1.5e-7",
                                "set 1 2 1e+21",
                                "set 1 2 123456789012345680000",
                                "set 1 2 -2.5",
                                "rows 2 0 [3 4]",
                                "rows 2 0 []",
                                "branch 1 0 0",
                                "attach 0 0 1",
                            }));

  CHECK(rejects({}));
  CHECK(rejects({2}));
  CHECK(rejects(Batch().u8(9).bytes));
  CHECK(rejects(Batch().u8(1).u32(1).bytes));
  CHECK(rejects(Batch().u8(4).u32(1).u32(0).u32(3).u32(1).bytes));
  CHECK(rejects(Batch().u8(2).u32(1).u32(0).u8(7).bytes));
  CHECK(rejects(Batch().u8(2).u32(1).u32(0).string("\xc0\xaf").bytes));
  CHECK(rejects(Batch().u8(2).u32(1).u32(0).string("\xed\xa0\x80").bytes));
}

const std::map<std::string, std::function<void()>> kCases = {
    {"timers", timers}, {"microtasks", microtasks}, {"console", console},
    {"text", text},     {"calls", calls},           {"session", session},
    {"refused", refused}, {"stable_rows", stable_rows}, {"ops", ops},
};

}  // namespace

int main(int argc, char **argv) {
  if (argc != 3 || !kCases.count(argv[2])) {
    std::fprintf(stderr, "usage: weft_core_tests <bundle dir> <case>\n");
    return 2;
  }
  dir = argv[1];
  try {
    kCases.at(argv[2])();
  } catch (const std::exception &e) {
    std::fprintf(stderr, "%s\n", e.what());
    return 1;
  }
  return 0;
}
