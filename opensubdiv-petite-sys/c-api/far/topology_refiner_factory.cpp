#include <opensubdiv/far/topologyDescriptor.h>
#include <opensubdiv/far/topologyRefiner.h>
#include <opensubdiv/far/topologyRefinerFactory.h>

typedef OpenSubdiv::Far::TopologyDescriptor TopologyDescriptor;
typedef OpenSubdiv::Far::TopologyRefiner TopologyRefiner;
typedef OpenSubdiv::Far::TopologyRefinerFactory<TopologyDescriptor>::Options Options;

extern "C"
{
    TopologyRefiner *TopologyRefinerFactory_TopologyDescriptor_Create(
        TopologyDescriptor *descriptor, Options options)
    {
        return OpenSubdiv::Far::TopologyRefinerFactory<TopologyDescriptor>::Create(
            *descriptor, options);
    }
}