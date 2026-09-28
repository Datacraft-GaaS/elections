# SPDX-License-Identifier: Apache-2.0
#
# eel BVR desktop client (Qt 6).
#
# The Rust side is built as a static library and linked in, so the binary has no
# runtime search path to configure and cannot end up paired with a mismatched
# shared library on an air-gapped station.
#
# Requires a Qt 6 installation with the Widgets module. The Rust bridge is
# built by cargo, not by CMake, so that the workspace's --locked and
# supply-chain rules apply to it:
#
#   cargo build -p eel-bvr-bridge --release --locked
#
# then configure and build this directory. EMBEDDED_WORKFLOW points at the
# shipped document and is compiled into the binary so a station works even if
# the file is absent from disk.

cmake_minimum_required(VERSION 3.21)
project(eel-bvr-desktop LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_INCLUDE_CURRENT_DIR ON)

find_package(Qt6 REQUIRED COMPONENTS Widgets)

# --- Rust bridge -----------------------------------------------------------
#
# Located from the workspace root so the build fails loudly with a useful
# message if the static library has not been built, rather than at link time
# with a wall of undefined symbols.
set(REPO_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../../../..")
get_filename_component(REPO_ROOT "${REPO_ROOT}" ABSOLUTE)

# The bridge is a workspace member, so cargo writes the archive to the shared
# workspace target directory. The crate-local path is checked too so this still
# works if the crate is ever built standalone.
set(BRIDGE_CANDIDATES
    "${REPO_ROOT}/target/release/libeel_bvr_bridge.a"
    "${CMAKE_CURRENT_SOURCE_DIR}/bridge/target/release/libeel_bvr_bridge.a"
)

set(BRIDGE_LIB "")
foreach(candidate IN LISTS BRIDGE_CANDIDATES)
    if(EXISTS "${candidate}")
        set(BRIDGE_LIB "${candidate}")
        break()
    endif()
endforeach()

if(NOT BRIDGE_LIB)
    set(SEARCHED "")
    foreach(candidate IN LISTS BRIDGE_CANDIDATES)
        set(SEARCHED "${SEARCHED}\n  ${candidate}")
    endforeach()
    message(FATAL_ERROR
        "Rust bridge static library not found.\n"
        "Build it first:\n"
        "  cargo build -p eel-bvr-bridge --release --locked\n"
        "Searched:${SEARCHED}")
endif()

message(STATUS "Using Rust bridge: ${BRIDGE_LIB}")

# --- embedded workflow document --------------------------------------------
set(STANDARD_WORKFLOW "${CMAKE_CURRENT_SOURCE_DIR}/workflows/standard-enrolment.json")
if(NOT EXISTS "${STANDARD_WORKFLOW}")
    message(FATAL_ERROR "Standard workflow document missing: ${STANDARD_WORKFLOW}")
endif()

# --- executable ------------------------------------------------------------
add_executable(eel-bvr-desktop
    src/main.cpp
    src/registration_window.cpp
    src/registration_window.h
    src/bvr_bridge.cpp
    src/bvr_bridge.h
)

target_include_directories(eel-bvr-desktop PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/src
    ${CMAKE_CURRENT_SOURCE_DIR}/bridge/include
)

target_compile_definitions(eel-bvr-desktop PRIVATE
    EMBEDDED_WORKFLOW="${STANDARD_WORKFLOW}"
)

target_link_libraries(eel-bvr-desktop PRIVATE
    Qt6::Widgets
    "${BRIDGE_LIB}"
)

# Rust's std needs these on any Unix host. Order matters for some linkers:
# the static archive must precede the system libraries that satisfy its
# undefined symbols.
if(UNIX)
    find_package(Threads REQUIRED)
    target_link_libraries(eel-bvr-desktop PRIVATE Threads::Threads ${CMAKE_DL_LIBS})
    if(APPLE)
        target_link_libraries(eel-bvr-desktop PRIVATE
            "-framework Security"
            "-framework CoreFoundation"
        )
    endif()
endif()

# Warnings are errors here: this is a client that handles citizen PII, and a
# silent narrowing conversion in a date or quality field is a data-integrity
# defect, not a style issue.
if(MSVC)
    target_compile_options(eel-bvr-desktop PRIVATE /W4 /WX)
else()
    target_compile_options(eel-bvr-desktop PRIVATE -Wall -Wextra -Wpedantic)
endif()
