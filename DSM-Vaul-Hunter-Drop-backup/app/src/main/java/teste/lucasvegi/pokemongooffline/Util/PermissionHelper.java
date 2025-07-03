package teste.lucasvegi.pokemongooffline.Util;

import android.Manifest;
import android.app.Activity;
import android.content.Intent;
import android.net.Uri;
import android.provider.Settings;


import com.google.android.material.dialog.MaterialAlertDialogBuilder;

import com.karumi.dexter.Dexter;
import com.karumi.dexter.MultiplePermissionsReport;
import com.karumi.dexter.PermissionToken;
import com.karumi.dexter.listener.PermissionDeniedResponse;
import com.karumi.dexter.listener.PermissionGrantedResponse;
import com.karumi.dexter.listener.PermissionRequest;
import com.karumi.dexter.listener.multi.MultiplePermissionsListener;
import com.karumi.dexter.listener.single.PermissionListener;

import java.util.ArrayList;
import java.util.List;

/**
 * Helper class for managing runtime permissions in Android
 */
public class PermissionHelper {
    private PermissionHelper() {
    }


    /**
     * Common interface for permission callback
     */
    public interface PermissionCallback {
        void onPermissionsGranted();
        void onPermissionsDenied();
    }

    /**
     * Check and request location permissions
     */
    public static void checkLocationPermissions(Activity activity, PermissionCallback callback) {
        List<String> permissions = new ArrayList<>();
        permissions.add(Manifest.permission.ACCESS_FINE_LOCATION);
        permissions.add(Manifest.permission.ACCESS_COARSE_LOCATION);

        Dexter.withContext(activity)
                .withPermissions(permissions)
                .withListener(new MultiplePermissionsListener() {
                    @Override
                    public void onPermissionsChecked(MultiplePermissionsReport report) {
                        if (report.areAllPermissionsGranted()) {
                            callback.onPermissionsGranted();
                        } else {
                            if (report.isAnyPermissionPermanentlyDenied()) {
                                showSettingsDialog(activity, "Location Permission",
                                        "This app needs location permissions to work properly. Please grant the permissions in Settings.");
                            }
                            callback.onPermissionsDenied();
                        }
                    }

                    @Override
                    public void onPermissionRationaleShouldBeShown(List<PermissionRequest> permissions, PermissionToken token) {
                        token.continuePermissionRequest();
                    }
                }).check();
    }

    /**
     * Check and request camera permissions
     */
    public static void checkCameraPermission(Activity activity, PermissionCallback callback) {
        Dexter.withContext(activity)
                .withPermission(Manifest.permission.CAMERA)
                .withListener(new PermissionListener() {
                    @Override
                    public void onPermissionGranted(PermissionGrantedResponse response) {
                        callback.onPermissionsGranted();
                    }

                    @Override
                    public void onPermissionDenied(PermissionDeniedResponse response) {
                        if (response.isPermanentlyDenied()) {
                            showSettingsDialog(activity, "Camera Permission",
                                    "This app needs camera permission to scan for Pokemon. Please grant the permission in Settings.");
                        }
                        callback.onPermissionsDenied();
                    }

                    @Override
                    public void onPermissionRationaleShouldBeShown(PermissionRequest permission, PermissionToken token) {
                        token.continuePermissionRequest();
                    }
                }).check();
    }

    /**
     * Show dialog to guide user to settings when permission is permanently denied
     */
    private static void showSettingsDialog(Activity activity, String title, String message) {
        new MaterialAlertDialogBuilder(activity)
                .setTitle(title)
                .setMessage(message)
                .setPositiveButton("Go to Settings", (dialog, which) -> {
                    dialog.dismiss();
                    openAppSettings(activity);
                })
                .setNegativeButton("Cancel", (dialog, which) -> dialog.dismiss())
                .show();
    }

    /**
     * Open app settings page
     */
    private static void openAppSettings(Activity activity) {
        Intent intent = new Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS);
        Uri uri = Uri.fromParts("package", activity.getPackageName(), null);
        intent.setData(uri);
        activity.startActivity(intent);
    }
}
