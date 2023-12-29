
file(GLOB_RECURSE SOURCE_FILES "${CMAKE_CURRENT_SOURCE_DIR}/*.c"
                               "${CMAKE_CURRENT_SOURCE_DIR}/*.cpp"
                               "${CMAKE_CURRENT_SOURCE_DIR}/*.py")

include_directories(${CMAKE_SOURCE_DIR}/include)

set(EXECUTABLE_OUTPUT_PATH ${CMAKE_CURRENT_BINARY_DIR})

foreach(SRC_FILE ${SOURCE_FILES})
    get_filename_component(BASENAME ${SRC_FILE} NAME_WE)
    file(RELATIVE_PATH SOURCE_DIR ${CMAKE_CURRENT_SOURCE_DIR} ${SRC_FILE})
    get_filename_component(BIN_DIR ${SOURCE_DIR} DIRECTORY)
    
    get_filename_component(FILE_SUFFIX ${SRC_FILE} EXT)
    string(TOLOWER ${FILE_SUFFIX} FILE_SUFFIX)

    if (${FILE_SUFFIX} STREQUAL ".c" OR ${FILE_SUFFIX} STREQUAL ".cpp")
        add_executable(${BASENAME} ${SRC_FILE})
        set_property(TARGET ${BASENAME} PROPERTY RUNTIME_OUTPUT_DIRECTORY ${EXECUTABLE_OUTPUT_PATH}/${BIN_DIR})
        install(TARGETS ${BASENAME} DESTINATION ${BIN_DIR})
    elseif (${FILE_SUFFIX} STREQUAL ".py")
        install(FILES ${SRC_FILE} DESTINATION ${BIN_DIR})
    endif()
endforeach()
