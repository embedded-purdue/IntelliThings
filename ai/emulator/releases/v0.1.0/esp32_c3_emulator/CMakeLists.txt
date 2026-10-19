# The following five lines of boilerplate have to be in your project's
# CMakeLists in this exact order for cmake to work correctly
cmake_minimum_required(VERSION 3.22)

# Credentials live in the gitignored sdkconfig.local (template: sdkconfig.local.example).
# Defaults only fill in options that aren't already in sdkconfig — delete sdkconfig after
# editing sdkconfig.local, or use `idf.py menuconfig`.
set(SDKCONFIG_DEFAULTS "sdkconfig.defaults")
if(EXISTS "${CMAKE_CURRENT_LIST_DIR}/sdkconfig.local")
    list(APPEND SDKCONFIG_DEFAULTS "sdkconfig.local")
endif()

include($ENV{IDF_PATH}/tools/cmake/project.cmake)
project(esp32_c3_emulator)
