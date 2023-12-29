#pragma once

#include <stdint.h>

namespace plugin
{
    class IInterface;
    class IPlugin;
}

extern "C"
{
    void interface_shutdown(plugin::IInterface *);
    const char *interface_get_name(plugin::IInterface *);
    uint64_t interface_get_frame(plugin::IInterface *);
    double interface_get_position_x(plugin::IInterface *);
    double interface_get_position_y(plugin::IInterface *);
    double interface_get_position_z(plugin::IInterface *);
}