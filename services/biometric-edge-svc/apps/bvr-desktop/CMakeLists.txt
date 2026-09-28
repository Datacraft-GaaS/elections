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
# Built by cargo, not CMake, so the workspace's --locked and supply-chain rules
# apply. CMake drives it via a custom target rather than expecting a developer to
# run cargo by hand first: a build step that only works if you happened to know
# an undocumented prerequisite is a build step that gets skipped.
get_filename_component(REPO_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../../../.." ABSOLUTE)

set(BRIDGE_LIB "${REPO_ROOT}/target/${CMAKE_BUILD_TYPE}/libeel_bvr_bridge.a")
if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(BRIDGE_LIB "${REPO_ROOT}/target/debug/libeel_bvr_bridge.a")
else()
    set(BRIDGE_LIB "${REPO_ROOT}/target/release/libeel_bvr_bridge.a")
endif()

find_program(CARGO_EXECUTABLE cargo REQUIRED)
message(STATUS "Cargo: ${CARGO_EXECUTABLE}")

if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(CARGO_PROFILE_FLAG "")
else()
    set(CARGO_PROFILE_FLAG "--release")
endif()

add_custom_command(
    OUTPUT "${BRIDGE_LIB}"
    COMMAND ${CMAKE_COMMAND} -E env
            "CARGO_TARGET_DIR=${REPO_ROOT}/target"
            ${CARGO_EXECUTABLE} build -p eel-bvr-bridge ${CARGO_PROFILE_FLAG} --locked
    WORKING_DIRECTORY "${REPO_ROOT}"
    COMMENT "Building the Rust bridge (eel-bvr-bridge)"
    VERBATIM
)
add_custom_target(eel_bvr_bridge_build DEPENDS "${BRIDGE_LIB}")
message(STATUS "Rust bridge: ${BRIDGE_LIB}")

# --- embedded workflow document --------------------------------------------
#
# The standard workflow is compiled into the binary, not merely pointed at, so
# a station works even if the file is absent from disk. It becomes a C++ raw
# string literal, which avoids having to escape JSON.
set(STANDARD_WORKFLOW "${CMAKE_CURRENT_SOURCE_DIR}/workflows/standard-enrolment.json")
if(NOT EXISTS "${STANDARD_WORKFLOW}")
    message(FATAL_ERROR "Standard workflow document missing: ${STANDARD_WORKFLOW}")
endif()
file(READ "${STANDARD_WORKFLOW}" STANDARD_WORKFLOW_CONTENT)

# A raw string literal ends at the delimiter, so reject content that would close
# it early rather than emitting a file that does not compile.
string(FIND "${STANDARD_WORKFLOW_CONTENT}" ")EELWF\"" DELIMITER_POSITION)
if(NOT DELIMITER_POSITION EQUAL -1)
    message(FATAL_ERROR
        "Workflow document contains the raw-string delimiter and cannot be embedded")
endif()

configure_file(
    "${CMAKE_CURRENT_SOURCE_DIR}/src/embedded_workflow.h.in"
    "${CMAKE_CURRENT_BINARY_DIR}/embedded_workflow.h"
    @ONLY
)

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
    ${CMAKE_CURRENT_BINARY_DIR}
    ${CMAKE_CURRENT_SOURCE_DIR}/bridge/include
)

target_link_libraries(eel-bvr-desktop PRIVATE
    Qt6::Widgets
    "${BRIDGE_LIB}"
)

# Build the bridge before anything that links it.
add_dependencies(eel-bvr-desktop eel_bvr_bridge_build)

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
