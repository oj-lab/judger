set(TESTLIB_COMMIT "0.9.12")
set(TESTLIB_SHA256 "f2fdd835c66d2578a5b0a39bb7d7dfb7126b6fc216a401d110621e214e0df643")

file(DOWNLOAD "https://github.com/MikeMirzayanov/testlib/raw/${TESTLIB_COMMIT}/testlib.h"
    "${CMAKE_SOURCE_DIR}/include/testlib.h" 
    SHOW_PROGRESS
    EXPECTED_HASH SHA256=${TESTLIB_SHA256})
