#include <algorithm>
#include <unordered_map>
#include <vector>

#include "weft/runtime.h"

extern "C" size_t weft_stable_rows(const uint32_t *old_rows, size_t old_len,
                                   const uint32_t *new_rows, size_t new_len, uint8_t *stable) {
  std::fill(stable, stable + new_len, uint8_t{0});
  std::unordered_map<uint32_t, size_t> old_index;
  old_index.reserve(old_len);
  for (size_t i = 0; i < old_len; i++) old_index.emplace(old_rows[i], i);

  // Longest increasing subsequence of the old positions of reused rows
  // (patience sorting): `tails[k]` is the new index ending the best run of
  // length k + 1, `prev` links each row to its predecessor in that run.
  constexpr size_t kNone = size_t(-1);
  std::vector<size_t> position(new_len, kNone);
  std::vector<size_t> prev(new_len, kNone);
  std::vector<size_t> tails;
  for (size_t i = 0; i < new_len; i++) {
    auto found = old_index.find(new_rows[i]);
    if (found == old_index.end()) continue;
    position[i] = found->second;
    auto at = std::lower_bound(tails.begin(), tails.end(), position[i],
                               [&](size_t tail, size_t pos) { return position[tail] < pos; });
    if (at != tails.begin()) prev[i] = *(at - 1);
    if (at == tails.end()) {
      tails.push_back(i);
    } else {
      *at = i;
    }
  }
  for (size_t i = tails.empty() ? kNone : tails.back(); i != kNone; i = prev[i]) stable[i] = 1;
  return tails.size();
}
