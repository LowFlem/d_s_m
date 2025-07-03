package teste.lucasvegi.pokemongooffline.Util;

import android.app.Application;
import android.bluetooth.BluetoothSocket;
import android.content.Context;
import android.util.Log;

import androidx.annotation.NonNull;
import androidx.annotation.Nullable;
import androidx.multidex.MultiDexApplication;

/**
 * Application class that maintains global application state
 */
public class MyApp extends MultiDexApplication {

    private static final String TAG = "MyApp";
    private static Context appContext;
    private static BluetoothSocket bluetoothSocket;

    @Override
    public void onCreate() {
        super.onCreate();
        appContext = getApplicationContext();
        Log.d(TAG, "Application initialized");
        
        // Initialize database
        BancoDadosSingleton.getInstance().updateResourceIds();
        
        // Initialize OpenStreetMap
        MapConfigUtils.initializeOsmDroid(this);
    }

    /**
     * Get the application context from anywhere in the app
     * @return Application context
     */
    @NonNull
    public static Context getAppContext() {
        if (appContext == null) {
            throw new IllegalStateException("Application context is null");
        }
        return appContext;
    }

    /**
     * Get the current Bluetooth socket
     * @return Current Bluetooth socket or null if none
     */
    @Nullable
    public static BluetoothSocket getBluetoothSocket() {
        return bluetoothSocket;
    }

    /**
     * Set the current Bluetooth socket
     * @param socket Bluetooth socket to set
     */
    public static void setBluetoothSocket(BluetoothSocket socket) {
        bluetoothSocket = socket;
        Log.d(TAG, socket != null ? "Bluetooth socket set" : "Bluetooth socket cleared");
    }
    
    /**
     * Close and clear the Bluetooth socket if it exists
     */
    public static void closeBluetoothSocket() {
        if (bluetoothSocket != null) {
            try {
                bluetoothSocket.close();
                Log.d(TAG, "Bluetooth socket closed");
            } catch (Exception e) {
                Log.e(TAG, "Error closing Bluetooth socket", e);
            } finally {
                bluetoothSocket = null;
            }
        }
    }
    
    @Override
    public void onTerminate() {
        // Clean up resources
        closeBluetoothSocket();
        super.onTerminate();
    }
}
