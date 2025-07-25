# Install script for directory: /root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/hbr

# Set the install prefix
if(NOT DEFINED CMAKE_INSTALL_PREFIX)
  set(CMAKE_INSTALL_PREFIX "/root/repo/target/debug/build/opensubdiv-petite-sys-94eeb44b6ce262ab/out")
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
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/include/opensubdiv/hbr" TYPE FILE PERMISSIONS OWNER_READ GROUP_READ WORLD_READ FILES
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/hbr/allocator.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/hbr/bilinear.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/hbr/catmark.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/hbr/cornerEdit.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/hbr/creaseEdit.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/hbr/faceEdit.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/hbr/face.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/hbr/fvarData.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/hbr/fvarEdit.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/hbr/halfedge.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/hbr/hierarchicalEdit.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/hbr/holeEdit.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/hbr/loop.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/hbr/mesh.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/hbr/subdivision.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/hbr/vertexEdit.h"
    "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/hbr/vertex.h"
    )
endif()

