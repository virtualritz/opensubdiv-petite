#include <opensubdiv/far/stencilTableFactory.h>

extern "C"
{
    const OpenSubdiv::Far::StencilTable *StencilTableFactory_Create(
        OpenSubdiv::Far::TopologyRefiner *refiner,
        OpenSubdiv::Far::StencilTableFactory::Options options)
    {
        return OpenSubdiv::Far::StencilTableFactory::Create(*refiner, options);
    }
}