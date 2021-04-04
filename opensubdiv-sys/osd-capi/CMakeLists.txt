cmake_minimum_required(VERSION 3.5)
project(osd-capi)

set(CMAKE_CXX_STANDARD 14)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

set(SOURCES
    far/primvar_refiner.cpp
    far/stencil_table.cpp
    far/stencil_table_factory.cpp
    far/topology_refiner.cpp
    far/topology_level.cpp
    osd/cpu_vertex_buffer.cpp
    osd/cpu_evaluator.cpp
)

if(NOT DEFINED NO_CUDA)
    list(APPEND SOURCES
        osd/cuda_evaluator.cpp
        osd/cuda_vertex_buffer.cpp
    )
endif()

add_library(osd-capi STATIC
    ${SOURCES}
)

target_include_directories(osd-capi PUBLIC ${OSD_INCLUDE_PATH})
target_link_libraries(osd-capi PUBLIC ${LIB_OSD}/lib/libosdCPU.so ${LIB_OSD}/lib/libosdGPU.so)
install(TARGETS osd-capi DESTINATION ${CMAKE_INSTALL_PREFIX})