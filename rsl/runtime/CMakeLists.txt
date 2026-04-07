cmake_minimum_required(VERSION 3.31.6)
project(rsl-rt)

set(CXX_STANDARD_REQUIRED true)
set(CXX_STANDARD 23)

find_package(Vulkan REQUIRED volk)

set(REPO_DIR ${CMAKE_SOURCE_DIR}/../..)

add_library(rsl-rt
    
    src/main.cpp
    
)

target_include_directories(rsl-rt SYSTEM PUBLIC Vulkan_INCLUDE_DIRS)
target_include_directories(rsl-rt PUBLIC ${CMAKE_BINARY_DIR}/headers)
add_dependencies(rsl-rt Profiles)

target_link_libraries(rsl-rt Vulkan::volk)


add_executable(rt-test
    src/main-test.cpp
)
target_link_libraries(rt-test rsl-rt)


add_custom_target(Profiles DEPENDS ${CMAKE_BINARY_DIR}/headers/vulkan_profiles.hpp)
add_custom_command(
    OUTPUT ${CMAKE_BINARY_DIR}/headers/vulkan_profiles.hpp
    COMMAND python3 ${REPO_DIR}/util/gen_profiles_solution.py --registry ${REPO_DIR}/util/vk.xml --input ${REPO_DIR}/util/profiles --input-filenames rsl.json -c release --output-library-inc ${CMAKE_BINARY_DIR}/headers
    MAIN_DEPENDENCY ${REPO_DIR}/util/profiles/rsl.json
)


