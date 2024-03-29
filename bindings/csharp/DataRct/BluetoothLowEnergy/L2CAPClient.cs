using Windows.Devices.Enumeration;

namespace DataRct.BluetoothLowEnergy;

public class L2CAPClient : L2CapDelegate
{
    private readonly InternalNearbyServer _internalNearbyServer;

    public L2CAPClient(InternalNearbyServer internalNearbyServer)
    {
        _internalNearbyServer = internalNearbyServer;
    }
    
    public void OpenL2capConnection(string connectionId, string peripheralUuid, uint psm)
    {
    }
}