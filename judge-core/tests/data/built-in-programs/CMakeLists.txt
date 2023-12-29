cmake_minimum_required(VERSION 3.0)

project(judger-test VERSION 1.0)

set(CMAKE_C_COMPILER "gcc")
set(CMAKE_CXX_COMPILER "g++")

set(CMAKE_C_FLAGS "-Wall -O2 -std=gnu11 -static -lm")

if (CMAKE_COMPILER_IS_GNUCC)
    execute_process(COMMAND ${CMAKE_C_COMPILER} -dumpfullversion -dumpversion
            OUTPUT_VARIABLE GCC_VERSION)
    string(REGEX MATCHALL "[0-9]+" GCC_VERSION_COMPONENTS ${GCC_VERSION})
    list(GET GCC_VERSION_COMPONENTS 0 GCC_MAJOR)
    list(GET GCC_VERSION_COMPONENTS 1 GCC_MINOR)

    message(STATUS "CMAKE version is ${CMAKE_VERSION}")

    set(GCC_VERSION "${GCC_MAJOR}.${GCC_MINOR}")
    message(STATUS "GCC version is ${GCC_VERSION}")

    if (GCC_VERSION GREATER_EQUAL "11")
        set(CMAKE_CXX_FLAGS "-Wall -O2 -std=gnu++20 -static")
    else()
        set(CMAKE_CXX_FLAGS "-Wall -O2 -std=gnu++2a -static")
    endif()
endif()

include(include/testlib.cmake)

add_subdirectory(src)
