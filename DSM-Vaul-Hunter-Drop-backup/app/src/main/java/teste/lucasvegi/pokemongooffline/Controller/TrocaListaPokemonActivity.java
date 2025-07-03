package teste.lucasvegi.pokemongooffline.Controller;

import android.Manifest;
import android.annotation.SuppressLint;
import android.bluetooth.BluetoothAdapter;
import android.bluetooth.BluetoothDevice;
import android.bluetooth.BluetoothManager;
import android.bluetooth.BluetoothSocket;
import android.content.ContentValues;
import android.content.Context;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.location.Criteria;
import android.location.LocationManager;
import android.os.Build;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.os.Message;
import android.util.Log;
import android.view.View;
import android.widget.AdapterView;
import android.widget.Button;
import android.widget.ImageView;
import android.widget.ListView;
import android.widget.Toast;

import androidx.annotation.NonNull;
import androidx.appcompat.app.AppCompatActivity;
import androidx.core.app.ActivityCompat;
import androidx.core.content.ContextCompat;

import com.google.android.material.snackbar.Snackbar;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.util.List;
import java.util.UUID;

import teste.lucasvegi.pokemongooffline.Model.Aparecimento;
import teste.lucasvegi.pokemongooffline.Model.ControladoraFachadaSingleton;
import teste.lucasvegi.pokemongooffline.Model.Pokemon;
import teste.lucasvegi.pokemongooffline.Model.PokemonCapturado;
import teste.lucasvegi.pokemongooffline.R;
import teste.lucasvegi.pokemongooffline.Util.BancoDadosSingleton;
import teste.lucasvegi.pokemongooffline.Util.MyApp;
import teste.lucasvegi.pokemongooffline.View.AdapterTrocaPokemonsList;

/**
 * Activity for Pokemon trading selection and confirmation
 */
public class TrocaListaPokemonActivity extends AppCompatActivity implements AdapterView.OnItemClickListener {

    private static final String TAG = "TrocaListaPokemonActivity";
    private static final int REQUEST_LOCATION_PERMISSION = 1;
    protected static final UUID UUID_RFCOMM = UUID.fromString("00001101-0000-1000-8000-00805F9B34FB");

    private List<Pokemon> pokemons;
    private BluetoothAdapter bluetoothAdapter;
    private BluetoothSocket socket;
    private ConnectedThread connectedThread;

    // Message types for handler
    private interface MessageConstants {
        int MESSAGE_READ = 0;
        int MESSAGE_WRITE = 1;
        int MESSAGE_TOAST = 2;
    }

    // Handler for receiving messages from the Bluetooth thread
    private final Handler handler = new Handler(Looper.getMainLooper()) {
        @Override
        public void handleMessage(@NonNull Message msg) {
            switch (msg.what) {
                case MessageConstants.MESSAGE_WRITE:
                    byte[] writeBuf = (byte[]) msg.obj;
                    // Log sent message
                    Log.d(TAG, "Message sent: " + new String(writeBuf));
                    break;
                    
                case MessageConstants.MESSAGE_READ:
                    byte[] readBuf = (byte[]) msg.obj;
                    // Construct a string from the valid bytes in the buffer
                    String readMessage = new String(readBuf, 0, msg.arg1);
                    Log.d(TAG, "Message received: " + readMessage);
                    processMessage(readMessage);
                    break;
                    
                case MessageConstants.MESSAGE_TOAST:
                    Snackbar.make(findViewById(android.R.id.content), 
                            msg.getData().getString("toast", "Error in communication"),
                            Snackbar.LENGTH_SHORT).show();
                    break;
            }
        }
    };

    private ListView listView;
    private Pokemon ofertado = null;
    private Pokemon recebido = null;
    private boolean eu_aceitei = false;
    private boolean outro_aceitou = false;

    private Button aceitar;
    private Button rejeitar;
    private ImageView euAceitei;
    private ImageView outroAceitou;
    private ImageView meuPokemonSelecionado;
    private ImageView outroPokemonSelecionado;
    private AdapterTrocaPokemonsList adapterPokedex;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_troca_lista_pokemons);

        // Initialize Bluetooth adapter
        BluetoothManager bluetoothManager = (BluetoothManager) getSystemService(Context.BLUETOOTH_SERVICE);
        if (bluetoothManager != null) {
            bluetoothAdapter = bluetoothManager.getAdapter();
        }

        // Get view references
        initializeViews();
        
        // Check Bluetooth state
        checkBluetoothState();
        
        // Get Bluetooth socket from application and initialize connection
        initializeBluetoothConnection();
    }
    
    private void initializeViews() {
        try {
            // Set up Pokemon list
            pokemons = ControladoraFachadaSingleton.getInstance().getPokemons();
            listView = findViewById(R.id.listaTrocaPokemons);

            adapterPokedex = new AdapterTrocaPokemonsList(pokemons, this);
            listView.setAdapter(adapterPokedex);
            listView.setOnItemClickListener(this);

            // Get UI elements
            aceitar = findViewById(R.id.botaoAceitar);
            rejeitar = findViewById(R.id.botaoRejeitar);
            euAceitei = findViewById(R.id.euAceitei);
            outroAceitou = findViewById(R.id.outroAceitou);
            meuPokemonSelecionado = findViewById(R.id.meu_pokemon_selecionado);
            outroPokemonSelecionado = findViewById(R.id.outro_pokemon_selecionado);
            
            // Initialize UI state
            rejeitar.setEnabled(false);
            euAceitei.setImageResource(android.R.drawable.checkbox_off_background);
            outroAceitou.setImageResource(android.R.drawable.checkbox_off_background);
        } catch (Exception e) {
            Log.e(TAG, "Error initializing views: " + e.getMessage(), e);
            Toast.makeText(this, "Error initializing Pokemon trading", Toast.LENGTH_SHORT).show();
            finish();
        }
    }
    
    @SuppressLint("MissingPermission")
    private void checkBluetoothState() {
        if (bluetoothAdapter == null) {
            Toast.makeText(this, "Bluetooth is not supported on this device", Toast.LENGTH_SHORT).show();
            finish();
            return;
        }
        
        if (!bluetoothAdapter.isEnabled()) {
            Toast.makeText(this, "Bluetooth must be enabled for trading", Toast.LENGTH_SHORT).show();
            finish();
        }
    }
    
    private void initializeBluetoothConnection() {
        socket = MyApp.getBluetoothSocket();

        if (socket != null && socket.isConnected()) {
            Log.d(TAG, "Bluetooth socket found and connected");
            connectedThread = new ConnectedThread(socket);
            connectedThread.start();
        } else {
            Log.e(TAG, "No valid Bluetooth connection found");
            Toast.makeText(this, "No Bluetooth connection found", Toast.LENGTH_SHORT).show();
            finish();
        }
    }

    @Override
    public void onItemClick(AdapterView<?> parent, View view, int position, long id) {
        try {
            // Handle click on a Pokemon in the list
            ofertado = (Pokemon) parent.getAdapter().getItem(position);

            // Update UI to show selected Pokemon
            meuPokemonSelecionado.setImageResource(ofertado.getIcone());
            adapterPokedex.setSelected(position);

            // Send selection to other device
            String message = "CHANGE " + ofertado.getNumero();
            connectedThread.write(message.getBytes());

            // Reset trade agreement state
            rejeitarTroca(false);
        } catch (Exception e) {
            Log.e(TAG, "Error handling Pokemon selection: " + e.getMessage(), e);
            Toast.makeText(this, "Error selecting Pokemon", Toast.LENGTH_SHORT).show();
        }
    }

    public void aceitarTroca(View v) {
        aceitarTroca();
    }

    private void aceitarTroca() {
        // Validate that both parties have selected a Pokemon
        if (ofertado == null) {
            Snackbar.make(findViewById(android.R.id.content),
                    "You need to select a Pokemon to offer first!",
                    Snackbar.LENGTH_SHORT).show();
            return;
        }
        
        if (recebido == null) {
            Snackbar.make(findViewById(android.R.id.content),
                    "Wait for the other trainer to offer a Pokemon first!",
                    Snackbar.LENGTH_SHORT).show();
            return;
        }

        // Update local trade agreement state
        eu_aceitei = true;
        aceitar.setEnabled(false);
        rejeitar.setEnabled(true);
        adapterPokedex.setAreAllEnabled(false);
        euAceitei.setImageResource(android.R.drawable.checkbox_on_background);

        // Send acceptance to other device
        connectedThread.write("ACCEPT".getBytes());

        // If both parties have accepted, perform the trade
        if (outro_aceitou) {
            fazTroca();
        }
    }

    public void rejeitarTroca(View v) {
        rejeitarTroca(true);
    }

    private void rejeitarTroca(boolean sendMsg) {
        // Reset local trade agreement state
        eu_aceitei = false;
        outro_aceitou = false;
        aceitar.setEnabled(true);
        rejeitar.setEnabled(false);
        euAceitei.setImageResource(android.R.drawable.checkbox_off_background);
        outroAceitou.setImageResource(android.R.drawable.checkbox_off_background);
        adapterPokedex.setAreAllEnabled(true);
        adapterPokedex.notifyDataSetChanged();

        // Send rejection to other device if requested
        if (sendMsg) {
            connectedThread.write("REJECT".getBytes());
        }
    }

    private void fazTroca() {
        if (!eu_aceitei || !outro_aceitou) {
            rejeitarTroca(false);
            return;
        }

        // Check location permission
        if (ContextCompat.checkSelfPermission(this, Manifest.permission.ACCESS_FINE_LOCATION) 
                != PackageManager.PERMISSION_GRANTED) {
            ActivityCompat.requestPermissions(this,
                    new String[]{Manifest.permission.ACCESS_FINE_LOCATION},
                    REQUEST_LOCATION_PERMISSION);
            return;
        }
        
        performTradeWithLocation();
    }
    
    @SuppressLint("MissingPermission")
    private void performTradeWithLocation() {
        try {
            // Get current location
            LocationManager locationManager = (LocationManager) getSystemService(Context.LOCATION_SERVICE);
            if (locationManager == null) {
                Toast.makeText(this, "Location service not available", Toast.LENGTH_SHORT).show();
                return;
            }
            
            // Set location criteria
            Criteria criteria = new Criteria();
            if (getPackageManager().hasSystemFeature(PackageManager.FEATURE_LOCATION_GPS)) {
                criteria.setAccuracy(Criteria.ACCURACY_FINE);
            } else {
                criteria.setAccuracy(Criteria.ACCURACY_COARSE);
            }
            
            // Get provider
            String provider = locationManager.getBestProvider(criteria, true);
            if (provider == null) {
                Toast.makeText(this, "No location provider available", Toast.LENGTH_SHORT).show();
                return;
            }
            
            // Get last known location
            android.location.Location location = locationManager.getLastKnownLocation(provider);
            if (location == null) {
                Toast.makeText(this, "Could not get your location", Toast.LENGTH_SHORT).show();
                return;
            }
            
            // Process the received Pokemon
            double lat = location.getLatitude();
            double lon = location.getLongitude();
            Aparecimento ap = new Aparecimento();
            ap.setLatitude(lat);
            ap.setLongitude(lon);
            ap.setPokemon(recebido);
            ControladoraFachadaSingleton.getInstance().getUsuario().capturar(ap);
            
            // Mark the offered Pokemon as blocked
            PokemonCapturado paraEditar = null;
            for (PokemonCapturado capt : ControladoraFachadaSingleton.getInstance().getUsuario().getPokemons().get(ofertado)) {
                if (capt.getEstaBloqueado() == 0) {
                    capt.setEstaBloqueado(1);
                    paraEditar = capt;
                    break;
                }
            }
            
            if (paraEditar != null) {
                // Update database
                ContentValues valores = new ContentValues();
                valores.put("estaBloqueado", 1);
                String where = "login = '" + ControladoraFachadaSingleton.getInstance().getUsuario().getLogin() + "' AND " +
                        "idPokemon = " + ofertado.getNumero() + " AND " +
                        "dtCaptura = '" + paraEditar.getDtCaptura() + "'";
                BancoDadosSingleton.getInstance().atualizar("pokemonusuario", valores, where);
                Log.d(TAG, "Pokemon marked as traded");
            }
            
            // Show success message and finish activity
            Snackbar.make(findViewById(android.R.id.content), 
                    "Trade completed successfully!", 
                    Snackbar.LENGTH_SHORT).show();
            
            // Play success sound
            playTradeSoundAndFinish();
        } catch (Exception e) {
            Log.e(TAG, "Error performing trade: " + e.getMessage(), e);
            Toast.makeText(this, "Error completing trade", Toast.LENGTH_SHORT).show();
        }
    }
    
    private void playTradeSoundAndFinish() {
        new Handler(Looper.getMainLooper()).postDelayed(this::finish, 1500);
    }

    @Override
    public void onRequestPermissionsResult(int requestCode, @NonNull String[] permissions, @NonNull int[] grantResults) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults);
        if (requestCode == REQUEST_LOCATION_PERMISSION) {
            if (grantResults.length > 0 && grantResults[0] == PackageManager.PERMISSION_GRANTED) {
                performTradeWithLocation();
            } else {
                Toast.makeText(this, "Location permission is required for trading", Toast.LENGTH_SHORT).show();
                rejeitarTroca(true);
            }
        }
    }

    public void clickVoltar(View v) {
        finish();
    }

    @Override
    protected void onDestroy() {
        // Clean up resources
        if (connectedThread != null) {
            connectedThread.cancel();
            connectedThread = null;
        }
        
        super.onDestroy();
    }

    /**
     * Thread for handling Bluetooth communication
     */
    private class ConnectedThread extends Thread {
        private final BluetoothSocket mmSocket;
        private final InputStream mmInStream;
        private final OutputStream mmOutStream;
        private byte[] mmBuffer;

        public ConnectedThread(BluetoothSocket socket) {
            mmSocket = socket;
            InputStream tmpIn = null;
            OutputStream tmpOut = null;

            // Get the input and output streams
            try {
                tmpIn = socket.getInputStream();
            } catch (IOException e) {
                Log.e(TAG, "Error creating input stream", e);
            }
            
            try {
                tmpOut = socket.getOutputStream();
            } catch (IOException e) {
                Log.e(TAG, "Error creating output stream", e);
            }

            mmInStream = tmpIn;
            mmOutStream = tmpOut;
        }

        public void run() {
            mmBuffer = new byte[1024];
            int numBytes;

            Log.d(TAG, "Bluetooth connection thread running");

            // Keep listening to the InputStream until an exception occurs
            while (!isInterrupted()) {
                try {
                    // Read from the InputStream
                    numBytes = mmInStream.read(mmBuffer);
                    
                    // Send the obtained bytes to the UI activity
                    Message readMsg = handler.obtainMessage(
                            MessageConstants.MESSAGE_READ, numBytes, -1,
                            mmBuffer);
                    readMsg.sendToTarget();
                } catch (IOException e) {
                    Log.d(TAG, "Input stream disconnected", e);
                    break;
                }
            }
        }

        /**
         * Send data to the remote device
         */
        public void write(byte[] bytes) {
            try {
                mmOutStream.write(bytes);
                mmOutStream.flush();
                
                // Share the sent message with the UI activity
                Message writtenMsg = handler.obtainMessage(
                        MessageConstants.MESSAGE_WRITE, -1, -1, bytes);
                writtenMsg.sendToTarget();
            } catch (IOException e) {
                Log.e(TAG, "Error sending data", e);

                // Send a failure message back to the activity
                Message writeErrorMsg = handler.obtainMessage(MessageConstants.MESSAGE_TOAST);
                Bundle bundle = new Bundle();
                bundle.putString("toast", "Couldn't send data to the other device");
                writeErrorMsg.setData(bundle);
                handler.sendMessage(writeErrorMsg);
            }
        }

        /**
         * Shutdown the connection
         */
        public void cancel() {
            interrupt();
            try {
                mmSocket.close();
            } catch (IOException e) {
                Log.e(TAG, "Error closing Bluetooth socket", e);
            }
        }
    }

    /**
     * Process messages received from the other device
     */
    private void processMessage(String msg) {
        String[] flags = msg.split(" ");
        
        if (flags.length == 0) return;

        switch (flags[0]) {
            case "CHANGE":
                if (flags.length < 2) return;
                
                try {
                    // Parse received Pokemon number
                    int pokemonNumber = Integer.parseInt(flags[1]);
                    int position = pokemonNumber - 1;
                    
                    if (position >= 0 && position < pokemons.size()) {
                        // Get the received Pokemon
                        recebido = pokemons.get(position);
                        
                        // Update UI
                        runOnUiThread(() -> {
                            outroPokemonSelecionado.setImageResource(recebido.getIcone());
                            rejeitarTroca(false);
                        });
                    }
                } catch (NumberFormatException e) {
                    Log.e(TAG, "Invalid Pokemon number received: " + flags[1], e);
                }
                break;

            case "ACCEPT":
                runOnUiThread(() -> {
                    outro_aceitou = true;
                    outroAceitou.setImageResource(android.R.drawable.checkbox_on_background);
                    
                    if (eu_aceitei && outro_aceitou) {
                        fazTroca();
                    }
                });
                break;

            case "REJECT":
                runOnUiThread(() -> rejeitarTroca(false));
                break;
                
            default:
                Log.w(TAG, "Unknown message received: " + msg);
                break;
        }
    }
}
