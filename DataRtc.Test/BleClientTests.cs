using DataRct;
using DataRct.BluetoothLowEnergy;

namespace DataRtc.Test;

public class BleClientTests: DiscoveryDelegate
{
    [Fact]
    public async Task StartBleScanning()
    {
        var bleClient = new Discovery(this);
        bleClient.StartScan();

        await Task.Delay(30000000);
    }

    public void DeviceAdded(Device value)
    {
        throw new NotImplementedException();
    }

    public void DeviceRemoved(string deviceId)
    {
        throw new NotImplementedException();
    }
}