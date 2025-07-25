# Install script for directory: /root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv

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

if(NOT CMAKE_INSTALL_LOCAL_ONLY)
  # Include the install script for the subdirectory.
  include("/root/repo/target/debug/build/opensubdiv-petite-sys-773a59c5d5bc3b51/out/build/opensubdiv/hbr/cmake_install.cmake")
endif()

if(NOT CMAKE_INSTALL_LOCAL_ONLY)
  # Include the install script for the subdirectory.
  include("/root/repo/target/debug/build/opensubdiv-petite-sys-773a59c5d5bc3b51/out/build/opensubdiv/sdc/cmake_install.cmake")
endif()

if(NOT CMAKE_INSTALL_LOCAL_ONLY)
  # Include the install script for the subdirectory.
  include("/root/repo/target/debug/build/opensubdiv-petite-sys-773a59c5d5bc3b51/out/build/opensubdiv/vtr/cmake_install.cmake")
endif()

if(NOT CMAKE_INSTALL_LOCAL_ONLY)
  # Include the install script for the subdirectory.
  include("/root/repo/target/debug/build/opensubdiv-petite-sys-773a59c5d5bc3b51/out/build/opensubdiv/far/cmake_install.cmake")
endif()

if(NOT CMAKE_INSTALL_LOCAL_ONLY)
  # Include the install script for the subdirectory.
  include("/root/repo/target/debug/build/opensubdiv-petite-sys-773a59c5d5bc3b51/out/build/opensubdiv/bfr/cmake_install.cmake")
endif()

if(NOT CMAKE_INSTALL_LOCAL_ONLY)
  # Include the install script for the subdirectory.
  include("/root/repo/target/debug/build/opensubdiv-petite-sys-773a59c5d5bc3b51/out/build/opensubdiv/osd/cmake_install.cmake")
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/include/opensubdiv" TYPE FILE PERMISSIONS OWNER_READ GROUP_READ WORLD_READ FILES "/root/repo/opensubdiv-petite-sys/OpenSubdiv/opensubdiv/version.h")
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib" TYPE STATIC_LIBRARY FILES "/root/repo/target/debug/build/opensubdiv-petite-sys-773a59c5d5bc3b51/out/build/lib/libosdCPU.a")
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  include("/root/repo/target/debug/build/opensubdiv-petite-sys-773a59c5d5bc3b51/out/build/opensubdiv/CMakeFiles/osd_static_cpu.dir/install-cxx-module-bmi-Debug.cmake" OPTIONAL)
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  if(EXISTS "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/lib/libosdCPU.so.3.6.1" AND
     NOT IS_SYMLINK "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/lib/libosdCPU.so.3.6.1")
    file(RPATH_CHECK
         FILE "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/lib/libosdCPU.so.3.6.1"
         RPATH "/root/repo/target/debug/build/opensubdiv-petite-sys-773a59c5d5bc3b51/out/lib")
  endif()
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib" TYPE SHARED_LIBRARY FILES "/root/repo/target/debug/build/opensubdiv-petite-sys-773a59c5d5bc3b51/out/build/lib/libosdCPU.so.3.6.1")
  if(EXISTS "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/lib/libosdCPU.so.3.6.1" AND
     NOT IS_SYMLINK "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/lib/libosdCPU.so.3.6.1")
    file(RPATH_CHANGE
         FILE "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/lib/libosdCPU.so.3.6.1"
         OLD_RPATH "::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::"
         NEW_RPATH "/root/repo/target/debug/build/opensubdiv-petite-sys-773a59c5d5bc3b51/out/lib")
    if(CMAKE_INSTALL_DO_STRIP)
      execute_process(COMMAND "/usr/bin/strip" "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/lib/libosdCPU.so.3.6.1")
    endif()
  endif()
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib" TYPE SHARED_LIBRARY FILES "/root/repo/target/debug/build/opensubdiv-petite-sys-773a59c5d5bc3b51/out/build/lib/libosdCPU.so")
endif()

