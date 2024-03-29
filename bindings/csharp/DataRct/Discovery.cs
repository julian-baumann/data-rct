using DataRct.BluetoothLowEnergy;

namespace DataRct;

public class Discovery
{
    private readonly InternalDiscovery _internalHandler;
    private readonly BleClient _bleImplementation;

    public Discovery(DiscoveryDelegate discoveryDelegate)
    {
        _internalHandler = new InternalDiscovery(discoveryDelegate);
        _bleImplementation = new BleClient(discoveryDelegate, _internalHandler);
        
        _internalHandler.AddBleImplementation(_bleImplementation);
        
    }

    public void StartScan()
    {
        _internalHandler.Start();
    }

    public void StopScan()
    {
        _internalHandler.Stop();
    }
}