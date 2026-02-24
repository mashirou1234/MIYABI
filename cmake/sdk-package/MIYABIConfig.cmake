include_guard(GLOBAL)

set(MIYABI_VERSION "0.1.0")

get_filename_component(_MIYABI_PREFIX "${CMAKE_CURRENT_LIST_DIR}/.." ABSOLUTE)
set(_MIYABI_INCLUDE_DIR "${_MIYABI_PREFIX}/include")
set(_MIYABI_LIB_DIR "${_MIYABI_PREFIX}/lib")

set(_MIYABI_REQUIRED_FILES
    "${_MIYABI_INCLUDE_DIR}/miyabi/miyabi.h"
    "${_MIYABI_INCLUDE_DIR}/miyabi/bridge.h"
    "${_MIYABI_LIB_DIR}/libmiyabi_logic.a"
    "${_MIYABI_LIB_DIR}/libmiyabi_logic_cxx.a"
    "${_MIYABI_LIB_DIR}/libmiyabi_runtime.a"
    "${_MIYABI_LIB_DIR}/libbox2d.a"
)

foreach(_required_file IN LISTS _MIYABI_REQUIRED_FILES)
    if(NOT EXISTS "${_required_file}")
        message(FATAL_ERROR "MIYABI SDK is incomplete. Missing: ${_required_file}")
    endif()
endforeach()

if(NOT TARGET MIYABI::miyabi_logic)
    add_library(MIYABI::miyabi_logic STATIC IMPORTED)
    set_target_properties(MIYABI::miyabi_logic PROPERTIES
        IMPORTED_LOCATION "${_MIYABI_LIB_DIR}/libmiyabi_logic.a"
        INTERFACE_INCLUDE_DIRECTORIES "${_MIYABI_INCLUDE_DIR}"
    )
endif()

if(NOT TARGET MIYABI::miyabi_logic_cxx)
    add_library(MIYABI::miyabi_logic_cxx STATIC IMPORTED)
    set_target_properties(MIYABI::miyabi_logic_cxx PROPERTIES
        IMPORTED_LOCATION "${_MIYABI_LIB_DIR}/libmiyabi_logic_cxx.a"
        INTERFACE_INCLUDE_DIRECTORIES "${_MIYABI_INCLUDE_DIR}"
    )
endif()

if(NOT TARGET MIYABI::miyabi_runtime)
    add_library(MIYABI::miyabi_runtime STATIC IMPORTED)
    set_target_properties(MIYABI::miyabi_runtime PROPERTIES
        IMPORTED_LOCATION "${_MIYABI_LIB_DIR}/libmiyabi_runtime.a"
        INTERFACE_INCLUDE_DIRECTORIES "${_MIYABI_INCLUDE_DIR}"
    )
endif()

if(NOT TARGET MIYABI::box2d)
    add_library(MIYABI::box2d STATIC IMPORTED)
    set_target_properties(MIYABI::box2d PROPERTIES
        IMPORTED_LOCATION "${_MIYABI_LIB_DIR}/libbox2d.a"
        INTERFACE_INCLUDE_DIRECTORIES "${_MIYABI_INCLUDE_DIR}"
    )
endif()

if(NOT TARGET MIYABI::SDK)
    add_library(MIYABI::SDK INTERFACE IMPORTED)
    set_target_properties(MIYABI::SDK PROPERTIES
        INTERFACE_INCLUDE_DIRECTORIES "${_MIYABI_INCLUDE_DIR}"
        INTERFACE_LINK_LIBRARIES "MIYABI::miyabi_logic;MIYABI::miyabi_logic_cxx;MIYABI::miyabi_runtime;MIYABI::box2d"
        INTERFACE_LINK_OPTIONS "$<$<PLATFORM_ID:Darwin>:LINKER:-no_warn_duplicate_libraries>"
    )
endif()

set(MIYABI_INCLUDE_DIRS "${_MIYABI_INCLUDE_DIR}")
set(MIYABI_LIBRARIES MIYABI::SDK)
