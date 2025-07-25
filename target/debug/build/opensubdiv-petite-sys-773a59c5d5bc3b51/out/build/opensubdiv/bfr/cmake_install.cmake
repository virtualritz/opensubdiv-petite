# Install script for directory: /root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/bfr

# Set the install prefix
if(NOT DEFINED CMAKE_INSTALL_PREFIX)
  set(CMAKE_INSTALL_PREFIX "/root/repo/target/debug/build/opensubdiv-petite-sys-773a59c5d5bc3b51/out")
endif()
string(REGEX REPLACE "/$" "" CMAKE_INSTALL_PREFIX "${CMAKE_INSTALL_PREFIX}")

# Set the install configuration name.
if(NOT DEFINED CMAKE_INSTALL_CONFIG_NAME)
  if(BUILD_TYPE)
    string(REGEX REPLACE "^[^A-Za-z0-9_]+" ""
           CMAKE_INSTALL_CONFIG_NAME "${BUILD_TYPE}")
  else()
    set(CMAKE_INSTALL_CONFIG_NAME "Debug")
  endif()
  message(STATUS "Install configuration: \"${CMAKE_INSTALL_CONFIG_NAME}\"")
endif()

# Set the component getting installed.
if(NOT CMAKE_INSTALL_COMPONENT)
  if(COMPONENT)
    message(STATUS "Install component: \"${COMPONENT}\"")
    set(CMAKE_INSTALL_COMPONENT "${COMPONENT}")
  else()
    set(CMAKE_INSTALL_COMPONENT)
  endif()
endif()

# Install shared libraries without execute permission?
if(NOT DEFINED CMAKE_INSTALL_SO_NO_EXE)
  set(CMAKE_INSTALL_SO_NO_EXE "1")
endif()

# Is this installation the result of a crosscompile?
if(NOT DEFINED CMAKE_CROSSCOMPILING)
  set(CMAKE_CROSSCOMPILING "FALSE")
endif()

# Set default install directory permissions.
if(NOT DEFINED CMAKE_OBJDUMP)
  set(CMAKE_OBJDUMP "/usr/bin/objdump")
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/include/opensubdiv/bfr" TYPE FILE PERMISSIONS OWNER_READ GROUP_READ WORLD_READ FILES
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/bfr/irregularPatchType.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/bfr/limits.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/bfr/parameterization.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/bfr/refinerSurfaceFactory.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/bfr/surface.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/bfr/surfaceData.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/bfr/surfaceFactory.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/bfr/surfaceFactoryMeshAdapter.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/bfr/surfaceFactoryCache.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/bfr/tessellation.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/bfr/vertexDescriptor.h"
    )
endif()

