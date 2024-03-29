using DataRct;
using DataRct.BluetoothLowEnergy;

namespace DataRtc.Test;

public class BlePeripheralManagerTests
{
    [Fact]
    public async Task Test()
    {
        var manager = new BlePeripheralManager(null, null);
        manager.StartServer();


        await Task.Delay(30000000);
    }
}