#include <opensubdiv/far/topologyDescriptor.h>
#include <opensubdiv/far/topologyRefiner.h>
#include <opensubdiv/far/topologyRefinerFactory.h>

typedef OpenSubdiv::Far::TopologyRefiner TopologyRefiner;
typedef OpenSubdiv::Far::TopologyLevel TopologyLevel;
typedef OpenSubdiv::Far::TopologyDescriptor TopologyDescriptor;
typedef OpenSubdiv::Far::TopologyRefinerFactory<TopologyDescriptor>::Options Options;

extern "C"
{
    TopologyRefiner *TopologyRefinerFactory_TopologyDescriptor_Create(
        TopologyDescriptor *descriptor, Options options)
    {
        return OpenSubdiv::Far::TopologyRefinerFactory<TopologyDescriptor>::Create(
            *descriptor, options);
    }

    /// \brief Destroy a TopologyRefiner instance
    void TopologyRefiner_destroy(TopologyRefiner *refiner)
    {
        delete refiner;
    }

    /// \brief Returns the number of refinement levels
    int TopologyRefiner_GetNumLevels(TopologyRefiner *refiner)
    {
        return refiner->GetNumLevels();
    }

    /// \brief Returns the highest level of refinement
    int TopologyRefiner_GetMaxLevel(TopologyRefiner *refiner)
    {
        return refiner->GetMaxLevel();
    }

    /// \brief Returns the maximum vertex valence in all levels
    int TopologyRefiner_GetMaxValence(TopologyRefiner *refiner)
    {
        return refiner->GetMaxValence();
    }

    /// \brief Returns the total number of vertices in all levels
    int TopologyRefiner_GetNumVerticesTotal(TopologyRefiner *refiner)
    {
        return refiner->GetNumVerticesTotal();
    }

    /// \brief Returns the total number of edges in all levels
    int TopologyRefiner_GetNumEdgesTotal(TopologyRefiner *refiner)
    {
        return refiner->GetNumEdgesTotal();
    }

    /// \brief Returns the total number of edges in all levels
    int TopologyRefiner_GetNumFacesTotal(TopologyRefiner *refiner)
    {
        return refiner->GetNumFacesTotal();
    }

    /// \brief Returns the total number of face vertices in all levels
    int TopologyRefiner_GetNumFaceVerticesTotal(TopologyRefiner *refiner)
    {
        return refiner->GetNumFaceVerticesTotal();
    }

    /// \brief Returns a handle to access data specific to a particular level
    const TopologyLevel *TopologyRefiner_GetLevel(TopologyRefiner *refiner, int level)
    {
        return &refiner->GetLevel(level);
    }
}