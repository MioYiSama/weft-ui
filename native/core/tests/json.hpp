// A minimal JSON reader for the test hosts: layouts and conformance messages.

#pragma once

#include <cstdint>
#include <cstdlib>
#include <map>
#include <memory>
#include <stdexcept>
#include <string>
#include <string_view>
#include <variant>
#include <vector>

namespace json {

struct Value;
using Array = std::vector<Value>;
using Object = std::map<std::string, Value, std::less<>>;

struct Value {
  std::variant<std::nullptr_t, bool, double, std::string, std::shared_ptr<Array>, std::shared_ptr<Object>> data;

  bool is_null() const { return std::holds_alternative<std::nullptr_t>(data); }
  bool is_bool() const { return std::holds_alternative<bool>(data); }
  bool is_number() const { return std::holds_alternative<double>(data); }
  bool is_string() const { return std::holds_alternative<std::string>(data); }

  bool boolean() const { return get<bool>("a boolean"); }
  double number() const { return get<double>("a number"); }
  uint32_t u32() const { return uint32_t(number()); }
  const std::string &string() const { return get<std::string>("a string"); }
  const Array &array() const { return *get<std::shared_ptr<Array>>("an array"); }
  const Object &object() const { return *get<std::shared_ptr<Object>>("an object"); }

  const Value &operator[](std::string_view key) const {
    auto &members = object();
    auto found = members.find(key);
    if (found == members.end()) throw std::runtime_error("json: missing key " + std::string(key));
    return found->second;
  }

  bool has(std::string_view key) const { return object().count(key) > 0; }

 private:
  template <typename T>
  const T &get(const char *what) const {
    if (auto *v = std::get_if<T>(&data)) return *v;
    throw std::runtime_error(std::string("json: expected ") + what);
  }
};

class Parser {
 public:
  explicit Parser(std::string_view text) : text_(text) {}

  Value parse() {
    Value value = parse_value();
    skip();
    if (at_ != text_.size()) fail("trailing characters");
    return value;
  }

 private:
  std::string_view text_;
  size_t at_ = 0;

  [[noreturn]] void fail(const std::string &why) {
    throw std::runtime_error("json: " + why + " at offset " + std::to_string(at_));
  }

  void skip() {
    while (at_ < text_.size() && (text_[at_] == ' ' || text_[at_] == '\n' || text_[at_] == '\r' || text_[at_] == '\t')) at_++;
  }

  char peek() {
    skip();
    if (at_ >= text_.size()) fail("unexpected end");
    return text_[at_];
  }

  void expect(char c) {
    if (peek() != c) fail(std::string("expected '") + c + "'");
    at_++;
  }

  bool literal(std::string_view word) {
    if (text_.substr(at_, word.size()) != word) return false;
    at_ += word.size();
    return true;
  }

  Value parse_value() {
    char c = peek();
    if (c == '{') return parse_object();
    if (c == '[') return parse_array();
    if (c == '"') return Value{parse_string()};
    if (literal("true")) return Value{true};
    if (literal("false")) return Value{false};
    if (literal("null")) return Value{nullptr};
    const char *start = text_.data() + at_;
    char *end = nullptr;
    double number = std::strtod(start, &end);
    if (end == start) fail("unexpected character");
    at_ += size_t(end - start);
    return Value{number};
  }

  Value parse_object() {
    auto members = std::make_shared<Object>();
    expect('{');
    if (peek() == '}') {
      at_++;
      return Value{members};
    }
    for (;;) {
      if (peek() != '"') fail("expected a key");
      std::string key = parse_string();
      expect(':');
      (*members)[key] = parse_value();
      if (peek() == ',') {
        at_++;
        continue;
      }
      expect('}');
      return Value{members};
    }
  }

  Value parse_array() {
    auto items = std::make_shared<Array>();
    expect('[');
    if (peek() == ']') {
      at_++;
      return Value{items};
    }
    for (;;) {
      items->push_back(parse_value());
      if (peek() == ',') {
        at_++;
        continue;
      }
      expect(']');
      return Value{items};
    }
  }

  uint32_t hex4() {
    if (at_ + 4 > text_.size()) fail("truncated escape");
    uint32_t value = 0;
    for (int i = 0; i < 4; i++) {
      char c = text_[at_++];
      value <<= 4;
      if (c >= '0' && c <= '9') value |= uint32_t(c - '0');
      else if (c >= 'a' && c <= 'f') value |= uint32_t(c - 'a' + 10);
      else if (c >= 'A' && c <= 'F') value |= uint32_t(c - 'A' + 10);
      else fail("bad escape");
    }
    return value;
  }

  static void utf8(std::string &out, uint32_t cp) {
    if (cp < 0x80) {
      out += char(cp);
    } else if (cp < 0x800) {
      out += char(0xc0 | (cp >> 6));
      out += char(0x80 | (cp & 0x3f));
    } else if (cp < 0x10000) {
      out += char(0xe0 | (cp >> 12));
      out += char(0x80 | ((cp >> 6) & 0x3f));
      out += char(0x80 | (cp & 0x3f));
    } else {
      out += char(0xf0 | (cp >> 18));
      out += char(0x80 | ((cp >> 12) & 0x3f));
      out += char(0x80 | ((cp >> 6) & 0x3f));
      out += char(0x80 | (cp & 0x3f));
    }
  }

  std::string parse_string() {
    expect('"');
    std::string out;
    for (;;) {
      if (at_ >= text_.size()) fail("unterminated string");
      char c = text_[at_++];
      if (c == '"') return out;
      if (c != '\\') {
        out += c;
        continue;
      }
      if (at_ >= text_.size()) fail("unterminated string");
      char e = text_[at_++];
      switch (e) {
        case '"': out += '"'; break;
        case '\\': out += '\\'; break;
        case '/': out += '/'; break;
        case 'b': out += '\b'; break;
        case 'f': out += '\f'; break;
        case 'n': out += '\n'; break;
        case 'r': out += '\r'; break;
        case 't': out += '\t'; break;
        case 'u': {
          uint32_t cp = hex4();
          if (cp >= 0xd800 && cp < 0xdc00 && literal("\\u")) {
            uint32_t low = hex4();
            cp = 0x10000 + ((cp - 0xd800) << 10) + (low - 0xdc00);
          }
          utf8(out, cp);
          break;
        }
        default:
          fail("bad escape");
      }
    }
  }
};

inline Value parse(std::string_view text) { return Parser(text).parse(); }

}  // namespace json
