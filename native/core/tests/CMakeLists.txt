# hermesc flags of Weft bundles; keep them in sync with HERMESC_FLAGS in
# packages/vite-plugin/src/native.ts.
set(WEFT_HERMESC_FLAGS -O -emit-binary -Xes6-block-scoping -Xasync-generators -w)

# Kernel tests: each JS file under js/ is compiled by the hermesc of the
# Hermes subproject and run by one test case of weft_core_tests.
file(GLOB js_sources CONFIGURE_DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/js/*.js)
set(bundles)
foreach(source IN LISTS js_sources)
  get_filename_component(name ${source} NAME_WE)
  set(bundle ${CMAKE_CURRENT_BINARY_DIR}/js/${name}.hbc)
  add_custom_command(OUTPUT ${bundle}
    COMMAND ${CMAKE_COMMAND} -E make_directory ${CMAKE_CURRENT_BINARY_DIR}/js
    COMMAND hermesc ${WEFT_HERMESC_FLAGS} -out=${bundle} ${source}
    DEPENDS ${source} hermesc
    VERBATIM)
  list(APPEND bundles ${bundle})
endforeach()
add_custom_target(weft_core_test_bundles DEPENDS ${bundles})

add_executable(weft_core_tests kernel_test.cpp)
# Only the C ABI crosses the boundary of a shared kernel. Compile the C++
# ops helper into the test hosts, and place Windows DLLs beside each executable.
function(weft_test_host target)
  target_link_libraries(${target} PRIVATE weft::core)
  if(MSVC)
    target_compile_options(${target} PRIVATE /utf-8)
  endif()
  if(WEFT_CORE_SHARED)
    target_sources(${target} PRIVATE ${PROJECT_SOURCE_DIR}/src/ops.cpp)
  endif()
  if(WIN32 AND WEFT_CORE_SHARED)
    add_custom_command(TARGET ${target} POST_BUILD
      COMMAND ${CMAKE_COMMAND} -E copy_if_different
        $<TARGET_RUNTIME_DLLS:${target}> $<TARGET_FILE_DIR:${target}>
      COMMAND_EXPAND_LISTS
      VERBATIM)
  endif()
endfunction()
weft_test_host(weft_core_tests)
add_dependencies(weft_core_tests weft_core_test_bundles)
foreach(name timers microtasks console text calls session refused stable_rows ops)
  add_test(NAME core.${name} COMMAND weft_core_tests ${CMAKE_CURRENT_BINARY_DIR}/js ${name})
endforeach()

# Protocol conformance: conformance/native/build.ts bundles every fixture
# with @weft-ui/vite (including its hermesc step), and weft_conformance runs
# each bundle against a C++ renderer, comparing the ops log with the golden
# file the JS headless renderer wrote.
if(WEFT_CONFORMANCE)
  find_program(WEFT_NODE node REQUIRED)
  get_filename_component(repo ${PROJECT_SOURCE_DIR}/../.. ABSOLUTE)
  set(out ${CMAKE_CURRENT_BINARY_DIR}/conformance)
  add_executable(weft_conformance conformance.cpp)
  weft_test_host(weft_conformance)
  add_test(NAME conformance.build COMMAND ${WEFT_NODE} ${repo}/conformance/native/build.ts ${out})
  set_tests_properties(conformance.build PROPERTIES FIXTURES_SETUP conformance_bundles)
  file(GLOB goldens CONFIGURE_DEPENDS ${repo}/conformance/golden/*.ops)
  foreach(golden IN LISTS goldens)
    get_filename_component(name ${golden} NAME_WE)
    add_test(NAME conformance.${name} COMMAND weft_conformance ${out}/${name} ${golden})
    set_tests_properties(conformance.${name} PROPERTIES FIXTURES_REQUIRED conformance_bundles)
  endforeach()
endif()
