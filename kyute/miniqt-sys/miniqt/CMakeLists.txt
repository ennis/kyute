cmake_minimum_required(VERSION 3.10)
project(miniqt)

#==========================================================
# Config
set(CMAKE_CXX_STANDARD 11)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)

#==========================================================
# Dependencies
find_package(Qt5 COMPONENTS Widgets REQUIRED)

#==========================================================
# Sources
set(HEADERS miniqt.hpp miniqthelpers.hpp callback.hpp)
set(SOURCES callback.cpp miniqt.cpp)

add_library(miniqt ${SOURCES} ${HEADERS} ${RESOURCES})
target_link_libraries(miniqt PRIVATE Qt5::Widgets)
target_include_directories(miniqt PRIVATE .)
install(TARGETS miniqt DESTINATION .)

