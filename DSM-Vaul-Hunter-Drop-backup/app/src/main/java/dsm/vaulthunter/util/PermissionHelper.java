package dsm.vaulthunter.util;

import android.Manifest;
import android.app.Activity;
import android.content.Context;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.net.Uri;
import android.provider.Settings;
import android.util.Log;

import androidx.appcompat.app.AlertDialog;
import androidx.core.app.ActivityCompat;
import androidx.core.content.ContextCompat;

import teste.lucasvegi.pokemongooffline.R;

/**
 * Helper class for handling runtime permissions
 */
public class PermissionHelper {
    private static final String TAG = "PermissionHelper";
    
    private static final int CAMERA_PERMISSION_REQUEST_CODE = 100;
    private static final int LOCATION_PERMISSION_REQUEST_CODE = 101;
    private static final int STORAGE_PERMISSION_REQUEST_CODE = 102;
    private static final int BLUETOOTH_PERMISSION_REQUEST_CODE = 103;
    
    /**
     * Callback interface for permission results
     */
    public interface PermissionCallback {
        void onPermissionsGranted();
        void onPermissionsDenied();
    }
    
    /**
     * Check if camera permission is granted, and request it if not
     * @param activity Activity requesting the permission
     * @param callback Callback for permission result
     */
    public static void checkCameraPermission(Activity activity, PermissionCallback callback) {
        if (ContextCompat.checkSelfPermission(activity, Manifest.permission.CAMERA) 
                != PackageManager.PERMISSION_GRANTED) {
            
            // Check if we should show rationale
            if (ActivityCompat.shouldShowRequestPermissionRationale(activity, Manifest.permission.CAMERA)) {
                showPermissionRationaleDialog(
                        activity,
                        R.string.camera_permission_title,
                        R.string.camera_permission_rationale,
                        new String[] { Manifest.permission.CAMERA },
                        CAMERA_PERMISSION_REQUEST_CODE,
                        callback
                );
            } else {
                // No rationale needed, request directly
                ActivityCompat.requestPermissions(
                        activity,
                        new String[] { Manifest.permission.CAMERA },
                        CAMERA_PERMISSION_REQUEST_CODE
                );
            }
        } else {
            // Permission already granted
            callback.onPermissionsGranted();
        }
    }
    
    /**
     * Check if location permissions are granted, and request them if not
     * @param activity Activity requesting the permissions
     * @param callback Callback for permission result
     */
    public static void checkLocationPermissions(Activity activity, PermissionCallback callback) {
        String[] locationPermissions = {
                Manifest.permission.ACCESS_FINE_LOCATION,
                Manifest.permission.ACCESS_COARSE_LOCATION
        };
        
        if (ContextCompat.checkSelfPermission(activity, Manifest.permission.ACCESS_FINE_LOCATION) 
                != PackageManager.PERMISSION_GRANTED) {
            
            // Check if we should show rationale
            if (ActivityCompat.shouldShowRequestPermissionRationale(activity, Manifest.permission.ACCESS_FINE_LOCATION)) {
                showPermissionRationaleDialog(
                        activity,
                        R.string.location_permission_title,
                        R.string.location_permission_rationale,
                        locationPermissions,
                        LOCATION_PERMISSION_REQUEST_CODE,
                        callback
                );
            } else {
                // No rationale needed, request directly
                ActivityCompat.requestPermissions(
                        activity,
                        locationPermissions,
                        LOCATION_PERMISSION_REQUEST_CODE
                );
            }
        } else {
            // Permissions already granted
            callback.onPermissionsGranted();
        }
    }
    
    /**
     * Check if storage permissions are granted, and request them if not
     * @param activity Activity requesting the permissions
     * @param callback Callback for permission result
     */
    public static void checkStoragePermissions(Activity activity, PermissionCallback callback) {
        // For Android 10+ (API 29+), we use scoped storage and don't need these permissions
        // but we still request them for backward compatibility with older devices
        String[] storagePermissions = {
                Manifest.permission.READ_EXTERNAL_STORAGE,
                Manifest.permission.WRITE_EXTERNAL_STORAGE
        };
        
        if (ContextCompat.checkSelfPermission(activity, Manifest.permission.READ_EXTERNAL_STORAGE) 
                != PackageManager.PERMISSION_GRANTED) {
            
            // Check if we should show rationale
            if (ActivityCompat.shouldShowRequestPermissionRationale(activity, Manifest.permission.READ_EXTERNAL_STORAGE)) {
                showPermissionRationaleDialog(
                        activity,
                        R.string.storage_permission_title,
                        R.string.storage_permission_rationale,
                        storagePermissions,
                        STORAGE_PERMISSION_REQUEST_CODE,
                        callback
                );
            } else {
                // No rationale needed, request directly
                ActivityCompat.requestPermissions(
                        activity,
                        storagePermissions,
                        STORAGE_PERMISSION_REQUEST_CODE
                );
            }
        } else {
            // Permissions already granted
            callback.onPermissionsGranted();
        }
    }
    
    /**
     * Check if Bluetooth permissions are granted, and request them if not
     * @param activity Activity requesting the permissions
     * @param callback Callback for permission result
     */
    public static void checkBluetoothPermissions(Activity activity, PermissionCallback callback) {
        // Different permissions for different Android versions
        String[] bluetoothPermissions;
        
        if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.S) {
            // Android 12+ (API 31+)
            bluetoothPermissions = new String[] {
                    Manifest.permission.BLUETOOTH_SCAN,
                    Manifest.permission.BLUETOOTH_CONNECT,
                    Manifest.permission.BLUETOOTH_ADVERTISE
            };
        } else {
            // Android 11 and below
            bluetoothPermissions = new String[] {
                    Manifest.permission.BLUETOOTH,
                    Manifest.permission.BLUETOOTH_ADMIN
            };
        }
        
        boolean allGranted = true;
        for (String permission : bluetoothPermissions) {
            if (ContextCompat.checkSelfPermission(activity, permission) 
                    != PackageManager.PERMISSION_GRANTED) {
                allGranted = false;
                break;
            }
        }
        
        if (!allGranted) {
            // Check if we should show rationale
            boolean shouldShowRationale = false;
            for (String permission : bluetoothPermissions) {
                if (ActivityCompat.shouldShowRequestPermissionRationale(activity, permission)) {
                    shouldShowRationale = true;
                    break;
                }
            }
            
            if (shouldShowRationale) {
                showPermissionRationaleDialog(
                        activity,
                        R.string.bluetooth_permission_title,
                        R.string.bluetooth_permission_rationale,
                        bluetoothPermissions,
                        BLUETOOTH_PERMISSION_REQUEST_CODE,
                        callback
                );
            } else {
                // No rationale needed, request directly
                ActivityCompat.requestPermissions(
                        activity,
                        bluetoothPermissions,
                        BLUETOOTH_PERMISSION_REQUEST_CODE
                );
            }
        } else {
            // Permissions already granted
            callback.onPermissionsGranted();
        }
    }
    
    /**
     * Process permission request results
     * @param requestCode The request code
     * @param permissions The requested permissions
     * @param grantResults The grant results
     * @param callback Callback for permission result
     * @return true if the request code was handled
     */
    public static boolean onRequestPermissionsResult(int requestCode, String[] permissions, 
                                                  int[] grantResults, PermissionCallback callback) {
        switch (requestCode) {
            case CAMERA_PERMISSION_REQUEST_CODE:
            case LOCATION_PERMISSION_REQUEST_CODE:
            case STORAGE_PERMISSION_REQUEST_CODE:
            case BLUETOOTH_PERMISSION_REQUEST_CODE:
                boolean allGranted = true;
                for (int result : grantResults) {
                    if (result != PackageManager.PERMISSION_GRANTED) {
                        allGranted = false;
                        break;
                    }
                }
                
                if (allGranted) {
                    callback.onPermissionsGranted();
                } else {
                    callback.onPermissionsDenied();
                }
                return true;
                
            default:
                return false;
        }
    }
    
    /**
     * Show a dialog explaining why a permission is needed
     */
    private static void showPermissionRationaleDialog(Activity activity, int titleResId, int messageResId,
                                                   String[] permissions, int requestCode,
                                                   PermissionCallback callback) {
        AlertDialog.Builder builder = new AlertDialog.Builder(activity);
        builder.setTitle(titleResId)
               .setMessage(messageResId)
               .setPositiveButton(android.R.string.ok, (dialog, which) -> {
                   ActivityCompat.requestPermissions(activity, permissions, requestCode);
               })
               .setNegativeButton(android.R.string.cancel, (dialog, which) -> {
                   callback.onPermissionsDenied();
               })
               .setCancelable(false)
               .show();
    }
    
    /**
     * Show a dialog prompting the user to open settings when permissions are permanently denied
     */
    public static void showSettingsDialog(Context context, int titleResId, int messageResId) {
        AlertDialog.Builder builder = new AlertDialog.Builder(context);
        builder.setTitle(titleResId)
               .setMessage(messageResId)
               .setPositiveButton(R.string.settings, (dialog, which) -> {
                   Intent intent = new Intent();
                   intent.setAction(Settings.ACTION_APPLICATION_DETAILS_SETTINGS);
                   Uri uri = Uri.fromParts("package", context.getPackageName(), null);
                   intent.setData(uri);
                   context.startActivity(intent);
               })
               .setNegativeButton(android.R.string.cancel, null)
               .show();
    }
}