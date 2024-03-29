using System.Collections.ObjectModel;
using System.Collections.Specialized;
using System.Diagnostics;
using System.Runtime.InteropServices.WindowsRuntime;
using Windows.Devices.Bluetooth;
using Windows.Devices.Bluetooth.Advertisement;
using Windows.Devices.Bluetooth.GenericAttributeProfile;
using Windows.Devices.Enumeration;
using Windows.Storage.Streams;
using Windows.UI.Core;
using WinRT;
using NotifyCollectionChangedEventArgs = ABI.System.Collections.Specialized.NotifyCollectionChangedEventArgs;

namespace DataRct.BluetoothLowEnergy;

internal class BleClient: BleDiscoveryImplementationDelegate
{
    private DiscoveryDelegate _discoveryDelegate;
    private readonly InternalDiscovery _internalHandler;

    private readonly DeviceWatcher? _deviceWatcher;
    private readonly ObservableCollection<DeviceInformation?> _knownDevices = [];

    private static readonly Guid BleServiceUuid = Guid.Parse(DataRctMethods.GetBleServiceUuid());
    private static readonly Guid BleCharacteristicUuid = Guid.Parse(DataRctMethods.GetBleCharacteristicUuid());
    
    public BleClient(DiscoveryDelegate discoveryDelegate, InternalDiscovery internalHandler)
    {
        _discoveryDelegate = discoveryDelegate;
        _internalHandler = internalHandler;

        _knownDevices.CollectionChanged += KnownDevicesOnCollectionChanged;
        
        _deviceWatcher = DeviceInformation.CreateWatcher(
            $"System.Devices.DevObjectType:=5 AND System.Devices.Aep.ProtocolId:=\"{{BB7BB05E-5972-42B5-94FC-76EAA7084D49}}\"",
            ["System.Devices.Aep.DeviceAddress", "System.Devices.Aep.IsConnected", "System.Devices.Aep.Bluetooth.Le.IsConnectable"],
            DeviceInformationKind.AssociationEndpoint);

        _deviceWatcher.Added += DeviceWatcher_Added;
        _deviceWatcher.Updated += DeviceWatcher_Updated;
        _deviceWatcher.Removed += DeviceWatcher_Removed;
    }

    private GattDeviceService? GetBleService(BluetoothLEDevice device)
    {
        var service = device.GetGattServicesForUuidAsync(BleServiceUuid).GetAwaiter()
            .GetResult()?
            .Services?
            .SingleOrDefault();

        return service;
    }

    private byte[]? ReadCharacteristic(GattDeviceService service)
    {
        var characteristic = service.GetCharacteristicsForUuidAsync(BleCharacteristicUuid).GetAwaiter().GetResult()?.Characteristics?.SingleOrDefault();
        if (characteristic is not null)
        {
            var value = characteristic.ReadValueAsync().GetAwaiter().GetResult();
            var result = new byte[value.Value.Length];
            DataReader.FromBuffer(value.Value).ReadBytes(result);

            return result;
        }

        return null;
    }

    public void StartScanning()
    {
        lock (this)
        {
            _knownDevices.Clear();
        }

        _deviceWatcher?.Start();
    }

    public void StopScanning()
    {
        _deviceWatcher?.Stop();
    }
    
    private void KnownDevicesOnCollectionChanged(object? sender, System.Collections.Specialized.NotifyCollectionChangedEventArgs e)
    {
        var devices = _knownDevices;
        if (e.Action != NotifyCollectionChangedAction.Add)
        {
            return;
        }

        if (e.NewItems is not null && e.NewItems.Count != 0)
        {
            var newDevices = e.NewItems.Cast<DeviceInformation>()?.Where(x => x is not null);
            foreach (var device in newDevices ?? [])
            {
                var bleDevice = BluetoothLEDevice.FromIdAsync(device.Id).GetAwaiter().GetResult();

                var service = GetBleService(bleDevice);

                if (service is not null)
                {
                    var value = ReadCharacteristic(service);

                    if (value is not null)
                    {
                        _internalHandler.ParseDiscoveryMessage(value, service.Device.DeviceId);
                    }
                }
            }
        }
    }

    private DeviceInformation? FindBluetoothLeDeviceDisplay(string id)
    {
        return _knownDevices.FirstOrDefault(bleDeviceDisplay => bleDeviceDisplay?.Id == id);
    }

    private void DeviceWatcher_Added(DeviceWatcher sender, DeviceInformation deviceInfo)
    {
        lock (this)
        {
            if (sender != _deviceWatcher || FindBluetoothLeDeviceDisplay(deviceInfo.Id) != null)
            {
                return;
            }
            
            _knownDevices.Add(deviceInfo);
        }
    }

    private async void DeviceWatcher_Updated(DeviceWatcher sender, DeviceInformationUpdate deviceInfoUpdate)
    {
        lock (this)
        {
            if (sender != _deviceWatcher)
            {
                return;
            }
            
            var deviceInfo = FindBluetoothLeDeviceDisplay(deviceInfoUpdate.Id);
            deviceInfo?.Update(deviceInfoUpdate);
        }
    }

    private async void DeviceWatcher_Removed(DeviceWatcher sender, DeviceInformationUpdate deviceInfoUpdate)
    {
        lock (this)
        {
            if (sender != _deviceWatcher) return;
            
            var deviceInfo = FindBluetoothLeDeviceDisplay(deviceInfoUpdate.Id);
            if (deviceInfo != null)
            {
                _knownDevices.Remove(deviceInfo);
            }
        }
    }
}