using Windows.Devices.Bluetooth;
using Windows.Devices.Bluetooth.GenericAttributeProfile;
using Windows.Devices.Enumeration;
using Windows.Storage.Streams;

namespace DataRct.BluetoothLowEnergy;

public class BlePeripheralManager : BleServerImplementationDelegate
{
    private readonly InternalNearbyServer _internalHandler;
    private readonly NearbyServerDelegate _nearbyServerDelegate;
    private GattServiceProvider _peripheralManager;
    
    private static readonly Guid BleServiceUuid = Guid.Parse(DataRctMethods.GetBleServiceUuid());
    private static readonly Guid BleCharacteristicUuid = Guid.Parse(DataRctMethods.GetBleCharacteristicUuid());

    public BlePeripheralManager(InternalNearbyServer internalHandler, NearbyServerDelegate nearbyServerDelegate)
    {
        _internalHandler = internalHandler;
        _nearbyServerDelegate = nearbyServerDelegate;
    }

    public void StartServer()
    {
        var result  = GattServiceProvider.CreateAsync(BleServiceUuid)
            .GetAwaiter()
            .GetResult();

        if (result.Error == BluetoothError.Success)
        {
            _peripheralManager = result.ServiceProvider;
            var characteristicParameters = new GattLocalCharacteristicParameters
            {
                CharacteristicProperties = GattCharacteristicProperties.Read,
                StaticValue = null,
                ReadProtectionLevel = GattProtectionLevel.Plain,
                UserDescription = "InterShare Windows"
            };
            
            var characteristic = _peripheralManager.Service.CreateCharacteristicAsync(BleCharacteristicUuid, characteristicParameters)
                .GetAwaiter()
                .GetResult();
        
            var advertisingParameters = new GattServiceProviderAdvertisingParameters
            {
                IsConnectable = true,
                IsDiscoverable = true
            };
            
            characteristic.Characteristic.ReadRequested += CharacteristicOnReadRequested;
            
            _peripheralManager.StartAdvertising(advertisingParameters);
        }
    }

    private void CharacteristicOnReadRequested(GattLocalCharacteristic sender, GattReadRequestedEventArgs args)
    {
        var deferral = args.GetDeferral();

        try
        {
            var writer = new DataWriter();
            writer.WriteBytes(_internalHandler.GetAdvertisementData());

            var request = args.GetRequestAsync().GetAwaiter().GetResult();
            request.RespondWithValue(writer.DetachBuffer());
            
            deferral.Complete();
        }
        catch (Exception e)
        {
            Console.WriteLine(e);
        }
    }

    public void StopServer()
    {
        _peripheralManager?.StopAdvertising();
    }
}