package teste.lucasvegi.pokemongooffline.Controller;

import android.Manifest;
import android.annotation.SuppressLint;
import android.bluetooth.BluetoothAdapter;
import android.bluetooth.BluetoothDevice;
import android.bluetooth.BluetoothManager;
import android.bluetooth.BluetoothServerSocket;
import android.bluetooth.BluetoothSocket;
import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;
import android.content.IntentFilter;
import android.graphics.Color;
import android.os.Build;
import android.os.Bundle;
import android.util.Log;
import android.view.View;
import android.widget.AdapterView;
import android.widget.Button;
import android.widget.ListView;
import android.widget.Toast;

import androidx.activity.result.ActivityResultLauncher;
import androidx.activity.result.contract.ActivityResultContracts;
import androidx.appcompat.app.AppCompatActivity;

import com.karumi.dexter.Dexter;
import com.karumi.dexter.MultiplePermissionsReport;
import com.karumi.dexter.PermissionToken;
import com.karumi.dexter.listener.PermissionRequest;
import com.karumi.dexter.listener.multi.MultiplePermissionsListener;

import java.io.IOException;
import java.util.ArrayList;
import java.util.List;
import java.util.UUID;

import teste.lucasvegi.pokemongooffline.R;
import teste.lucasvegi.pokemongooffline.Util.MyApp;
import teste.lucasvegi.pokemongooffline.View.AdapterTroca;

/**
 * Activity for managing Bluetooth connections for Pokemon trading
 */
public class VocalistUsuriousActivity extends AppCompatActivity implements AdapterView.OnItemClickListener {

    private static final String TAG = "TrocaUsuariosActivity";
    private static final int REQUEST_ENABLE_BT = 1;
    protected static final String NAME = "SERVIDOR_TROCAS";
    // Standard SerialPortService ID
    protected static final UUID UUID_RFCOMM = UUID.fromString("00001101-0000-1000-8000-00805F9B34FB");

    private BluetoothAdapter bluetoothAdapter;
    private List<BluetoothDevice> bluetoothDevices;
    private AdapterTroca adapterTroca;

    AcceptThread acceptThread;
    ConnectThread connectThread;

    // Bluetooth enable launcher
    private ActivityResultLauncher<Intent> bluetoothEnableLauncher;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_troca_lista_usuarios);

        // Initialize permission launchers
        setupPermissionLaunchers();
        
        // Initialize the device list
        bluetoothDevices = new ArrayList<>();
        
        // Setup ListView
        ListView listView = findViewById(R.id.bluetooth_user_list);
        adapterTroca = new AdapterTroca(bluetoothDevices, this);
        listView.setAdapter(adapterTroca);
        listView.setOnItemClickListener(this);
        
        // Initialize Bluetooth adapter
        BluetoothManager bluetoothManager = (BluetoothManager) getSystemService(Context.BLUETOOTH_SERVICE);
        if (bluetoothManager != null) {
            bluetoothAdapter = bluetoothManager.getAdapter();
        }
        
        // If Bluetooth is not supported, show error and exit
        if (bluetoothAdapter == null) {
            Toast.makeText(this, "Bluetooth is not supported on this device", Toast.LENGTH_LONG).show();
            finish();
            return;
        }
        
        // Check and request required permissions
        checkAndRequestBluetoothPermissions();
    }
    
    private void setupPermissionLaunchers() {
        // Setup permission request launcher
        // Permission request launcher
        ActivityResultLauncher<String[]> requestPermissionLauncher = registerForActivityResult(
                new ActivityResultContracts.RequestMultiplePermissions(),
                permissions -> {
                    boolean allGranted = true;
                    for (Boolean granted : permissions.values()) {
                        allGranted = allGranted && granted;
                    }

                    if (allGranted) {
                        initializeBluetooth();
                    } else {
                        Toast.makeText(this, "Bluetooth permissions are required for trading", Toast.LENGTH_LONG).show();
                        finish();
                    }
                });
        
        // Setup Bluetooth enable launcher
        bluetoothEnableLauncher = registerForActivityResult(
                new ActivityResultContracts.StartActivityForResult(),
                result -> {
                    if (result.getResultCode() == RESULT_OK) {
                        initializeBluetoothServer();
                    } else {
                        Toast.makeText(this, "Bluetooth is required for trading", Toast.LENGTH_LONG).show();
                        finish();
                    }
                });
    }
    
    private void checkAndRequestBluetoothPermissions() {
        // Permissions required for Bluetooth operations
        List<String> permissions = new ArrayList<>();
        
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            // Android 12+ permissions
            permissions.add(Manifest.permission.BLUETOOTH_SCAN);
            permissions.add(Manifest.permission.BLUETOOTH_CONNECT);
        } else {
            // Older Android versions
            permissions.add(Manifest.permission.BLUETOOTH);
            permissions.add(Manifest.permission.BLUETOOTH_ADMIN);
            permissions.add(Manifest.permission.ACCESS_FINE_LOCATION);
        }
        
        Dexter.withContext(this)
                .withPermissions(permissions)
                .withListener(new MultiplePermissionsListener() {
                    @Override
                    public void onPermissionsChecked(MultiplePermissionsReport report) {
                        if (report.areAllPermissionsGranted()) {
                            initializeBluetooth();
                        } else {
                            Toast.makeText(VocalistUsuriousActivity.this,
                                    "Bluetooth permissions are required for trading", 
                                    Toast.LENGTH_LONG).show();
                            finish();
                        }
                    }

                    @Override
                    public void onPermissionRationaleShouldBeShown(List<PermissionRequest> permissions, PermissionToken token) {
                        token.continuePermissionRequest();
                    }
                }).check();
    }

    @SuppressLint("MissingPermission")
    private void initializeBluetooth() {
        // Register for broadcasts when a device is discovered
        IntentFilter filter = new IntentFilter();
        filter.addAction(BluetoothDevice.ACTION_FOUND);
        filter.addAction(BluetoothAdapter.ACTION_DISCOVERY_STARTED);
        filter.addAction(BluetoothAdapter.ACTION_DISCOVERY_FINISHED);
        registerReceiver(receiver, filter);
        
        // Enable Bluetooth if not enabled
        if (!bluetoothAdapter.isEnabled()) {
            Intent enableBtIntent = new Intent(BluetoothAdapter.ACTION_REQUEST_ENABLE);
            bluetoothEnableLauncher.launch(enableBtIntent);
        } else {
            initializeBluetoothServer();
        }
    }
    
    @SuppressLint("MissingPermission")
    private void initializeBluetoothServer() {
        // Start the Bluetooth server thread
        acceptThread = new AcceptThread();
        acceptThread.start();
        
        // Set device discoverable
        Intent discoverableIntent = new Intent(BluetoothAdapter.ACTION_REQUEST_DISCOVERABLE);
        discoverableIntent.putExtra(BluetoothAdapter.EXTRA_DISCOVERABLE_DURATION, 300);
        startActivity(discoverableIntent);
        
        // Start discovery automatically
        updateBT(null);
    }

    private final BroadcastReceiver receiver = new BroadcastReceiver() {
        @SuppressLint("MissingPermission")
        public void onReceive(Context context, Intent intent) {
            String action = intent.getAction();
            if (BluetoothDevice.ACTION_FOUND.equals(action)) {
                // Discovery has found a device. Get the BluetoothDevice
                // object and its info from the Intent.
                BluetoothDevice device = intent.getParcelableExtra(BluetoothDevice.EXTRA_DEVICE);

                // Add the device to the list if it has a name
                if (device != null && device.getName() != null && !bluetoothDevices.contains(device)) {
                    bluetoothDevices.add(device);
                    adapterTroca.notifyDataSetChanged();
                }
            }
            
            Button btn = findViewById(R.id.buscar);
            if (BluetoothAdapter.ACTION_DISCOVERY_STARTED.equals(action)) {
                // At the start of the search, alert the user to the waiting period
                Toast.makeText(context, "Searching for devices...", Toast.LENGTH_LONG).show();
                btn.setEnabled(false);
                btn.setTextColor(Color.GRAY);
            } else if (BluetoothAdapter.ACTION_DISCOVERY_FINISHED.equals(action)) {
                // Re-enable the button and restore the original color
                btn.setEnabled(true);
                btn.setTextColor(Color.BLACK);
            }
        }
    };

    @SuppressLint("MissingPermission")
    public void updateBT(View v) {
        Button btn = findViewById(R.id.buscar);
        btn.setTextColor(Color.GRAY);
        btn.setEnabled(false);

        // Clear previous devices
        bluetoothDevices.clear();
        adapterTroca.notifyDataSetChanged();

        // Start discovery
        if (bluetoothAdapter.isDiscovering()) {
            bluetoothAdapter.cancelDiscovery();
        }
        bluetoothAdapter.startDiscovery();
    }

    public void clickVoltar(View v) {
        finish();
    }

    @Override
    protected void onDestroy() {
        // Unregister broadcast receiver
        try {
            unregisterReceiver(receiver);
        } catch (IllegalArgumentException e) {
            Log.e(TAG, "Receiver not registered", e);
        }

        // Cancel ongoing Bluetooth operations
        cancelBluetoothOperations();

        super.onDestroy();
    }
    
    @SuppressLint("MissingPermission")
    private void cancelBluetoothOperations() {
        // Cancel discovery if it's running
        if (bluetoothAdapter != null && bluetoothAdapter.isDiscovering()) {
            bluetoothAdapter.cancelDiscovery();
        }
        
        // Cancel server thread
        if (acceptThread != null) {
            acceptThread.cancel();
            acceptThread = null;
        }
        
        // Cancel client thread
        if (connectThread != null) {
            connectThread.cancel();
            connectThread = null;
        }
    }

    @SuppressLint("MissingPermission")
    @Override
    public void onItemClick(AdapterView<?> adapterView, View view, int idx, long id) {
        // Get the selected device
        BluetoothDevice device = bluetoothDevices.get(idx);

        // Cancel discovery because it's resource intensive
        if (bluetoothAdapter.isDiscovering()) {
            bluetoothAdapter.cancelDiscovery();
        }

        // Cancel previous connection
        if (connectThread != null) {
            connectThread.cancel();
        }

        // Start a new connection
        connectThread = new ConnectThread(device);
        connectThread.start();
        
        Toast.makeText(this, "Connecting to " + device.getName() + "...", Toast.LENGTH_SHORT).show();
    }

    /**
     * Thread for accepting incoming Bluetooth connections
     */
    private class AcceptThread extends Thread {
        private final BluetoothServerSocket mmServerSocket;

        @SuppressLint("MissingPermission")
        public AcceptThread() {
            // Use a temporary object that is later assigned to mmServerSocket
            // because mmServerSocket is final.
            BluetoothServerSocket tmp = null;
            try {
                // MY_UUID is the app's UUID string, also used by the client code.
                tmp = bluetoothAdapter.listenUsingRfcommWithServiceRecord(NAME, UUID_RFCOMM);
            } catch (IOException e) {
                Log.e(TAG, "Socket's listen() method failed", e);
            }
            mmServerSocket = tmp;
        }

        public void run() {
            BluetoothSocket socket = null;
            // Keep listening until exception occurs or a socket is returned.
            Log.d(TAG, "Running Bluetooth server thread");

            while (!isInterrupted()) {
                try {
                    socket = mmServerSocket.accept();

                    if (socket != null) {
                        // A connection was accepted. Create a final reference to the socket
                        // to use in the lambda expression
                        final BluetoothSocket finalSocket = socket;
                        runOnUiThread(() -> manageMyConnectedSocket(finalSocket));
                        mmServerSocket.close();
                        break;
                    }
                } catch (IOException e) {
                    Log.e(TAG, "Socket's accept() method failed", e);
                    break;
                }
            }
        }

        // Closes the connect socket and causes the thread to finish.
        public void cancel() {
            Log.d(TAG, "Closing Bluetooth server");
            interrupt();
            try {
                mmServerSocket.close();
            } catch (IOException e) {
                Log.e(TAG, "Could not close the server socket", e);
            }
        }
    }

    /**
     * Thread for connecting to a remote Bluetooth device
     */
    private class ConnectThread extends Thread {
        private final BluetoothSocket mmSocket;
        private final BluetoothDevice mmDevice;

        @SuppressLint("MissingPermission")
        public ConnectThread(BluetoothDevice device) {
            // Use a temporary object that is later assigned to mmSocket
            // because mmSocket is final.
            BluetoothSocket tmp = null;
            mmDevice = device;

            try {
                // Get a BluetoothSocket to connect with the given BluetoothDevice.
                // MY_UUID is the app's UUID string, also used in the server code.
                tmp = device.createRfcommSocketToServiceRecord(UUID_RFCOMM);
            } catch (IOException e) {
                Log.e(TAG, "Socket's create() method failed", e);
            }
            mmSocket = tmp;
        }

        @SuppressLint("MissingPermission")
        public void run() {
            // Cancel discovery because it otherwise slows down the connection.
            bluetoothAdapter.cancelDiscovery();

            Log.d(TAG, "Running Bluetooth client thread");

            try {
                // Connect to the remote device through the socket. This call blocks
                // until it succeeds or throws an exception.
                mmSocket.connect();
                
                // The connection attempt succeeded. Perform work associated with
                // the connection on the UI thread. The socket is final so it's safe
                // to use in the lambda.
                runOnUiThread(() -> manageMyConnectedSocket(mmSocket));
                
            } catch (IOException connectException) {
                // Unable to connect; close the socket and return.
                Log.e(TAG, "Could not connect to device", connectException);
                runOnUiThread(() -> Toast.makeText(VocalistUsuriousActivity.this,
                        "Failed to connect to " + mmDevice.getName(), 
                        Toast.LENGTH_SHORT).show());
                try {
                    mmSocket.close();
                } catch (IOException closeException) {
                    Log.e(TAG, "Could not close the client socket", closeException);
                }
            }
        }

        // Closes the client socket and causes the thread to finish.
        public void cancel() {
            Log.d(TAG, "Closing Bluetooth client");
            interrupt();
            try {
                mmSocket.close();
            } catch (IOException e) {
                Log.e(TAG, "Could not close the client socket", e);
            }
        }
    }

    /**
     * Handle established Bluetooth connection
     */
    private void manageMyConnectedSocket(BluetoothSocket socket) {
        Log.d(TAG, "Bluetooth connection established");

        // Cancel the accept thread as we have a connection
        if (acceptThread != null) {
            acceptThread.cancel();
            acceptThread = null;
        }

        // Save the socket in the application context
        MyApp.setBluetoothSocket(socket);

        // Navigate to Pokemon selection screen
        Intent intent = new Intent(this, TrocaListaPokemonActivity.class);
        startActivity(intent);
    }
}
