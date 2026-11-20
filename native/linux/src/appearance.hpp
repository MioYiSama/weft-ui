#pragma once

namespace weft {

/// The current XDG appearance preference; light when the portal is unavailable.
bool dark();

/// Subscribes on GTK's main thread for the lifetime of a session.
class Appearance {
 public:
  Appearance(void (*changed)(void *), void *data);
  Appearance(const Appearance &) = delete;
  Appearance &operator=(const Appearance &) = delete;
  ~Appearance();

  /// Drives the same appearance change as the portal in the screenshot harness.
  static void set_test_dark(bool dark);

 private:
  struct Portal;
  static Portal &portal();

  void (*changed_)(void *);
  void *data_;
};

}  // namespace weft
