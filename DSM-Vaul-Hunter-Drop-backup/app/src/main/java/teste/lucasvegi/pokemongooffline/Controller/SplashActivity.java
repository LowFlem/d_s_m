package teste.lucasvegi.pokemongooffline.Controller;

import android.annotation.SuppressLint;
import android.content.Intent;
import android.content.pm.PackageInfo;
import android.content.pm.PackageManager;
import android.graphics.Color;
import android.media.MediaPlayer;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.util.Log;
import android.view.WindowManager;
import android.webkit.WebView;
import android.widget.TextView;
import android.widget.Toast;

import androidx.annotation.NonNull;
import androidx.appcompat.app.AppCompatActivity;
import androidx.core.splashscreen.SplashScreen;

import com.google.android.material.snackbar.Snackbar;

import teste.lucasvegi.pokemongooffline.Model.ControladoraFachadaSingleton;
import teste.lucasvegi.pokemongooffline.R;
import teste.lucasvegi.pokemongooffline.Util.PermissionHelper;
import teste.lucasvegi.pokemongooffline.Util.PermissionHelper.PermissionCallback;

/**
 * Splash screen activity that shows app loading and handles initial permissions
 */
public class SplashActivity extends AppCompatActivity {

    private static final String TAG = "SplashActivity";
    private static final int SPLASH_TIMEOUT = 6000;
    
    private WebView webView;
    private TextView versionTextView;
    private MediaPlayer mediaPlayer;
    private boolean permissionChecksComplete = false;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        // Apply the SplashScreen API if available (Android 12+)
        if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.S) {
            SplashScreen splashScreen = SplashScreen.installSplashScreen(this);
            splashScreen.setKeepOnScreenCondition(() -> !permissionChecksComplete);
        }
        
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_splash);
        
        // Configure the initial view
        configureInitialView();
        
        // Check for required permissions
        checkRequiredPermissions();
    }

    @Override
    protected void onPause() {
        super.onPause();
        
        // Pause media when activity is not visible
        if (mediaPlayer != null && mediaPlayer.isPlaying()) {
            mediaPlayer.pause();
        }
    }

    @Override
    protected void onResume() {
        super.onResume();
        
        // Resume media if it exists
        if (mediaPlayer != null && !mediaPlayer.isPlaying() && permissionChecksComplete) {
            mediaPlayer.start();
        }
    }

    @Override
    protected void onDestroy() {
        // Clean up media resources
        releaseMediaPlayer();
        super.onDestroy();
    }

    /**
     * Configure the initial view components
     */
    private void configureInitialView() {
        // Keep screen on while splash is displayed
        getWindow().addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);
        
        // Configure WebView for loading animation
        webView = findViewById(R.id.loaderSplash);
        webView.loadUrl("file:///android_asset/loading.gif");
        webView.setBackgroundColor(Color.TRANSPARENT);
        
        // Display app version dynamically
        versionTextView = findViewById(R.id.versaoApp);
        
        try {
            PackageInfo packageInfo = getPackageManager().getPackageInfo(getPackageName(), 0);
            String version = packageInfo.versionName;
            versionTextView.setText(getString(R.string.version_format, version));
        } catch (PackageManager.NameNotFoundException e) {
            Log.e(TAG, "Error getting package info", e);
            versionTextView.setText(getString(R.string.version_unknown));
        }
    }

    /**
     * Check for all required permissions
     */
    private void checkRequiredPermissions() {
        // Check location permissions first
        PermissionHelper.checkLocationPermissions(this, new PermissionCallback() {
            @Override
            public void onPermissionsGranted() {
                // If location is granted, continue with opening sequence
                Log.d(TAG, "Location permission granted");
                configureOpeningSequence();
            }

            @Override
            public void onPermissionsDenied() {
                // If location is denied, show explanation and continue anyway
                Log.w(TAG, "Location permission denied");
                Snackbar.make(findViewById(android.R.id.content), 
                        "Location permission is needed for full functionality", 
                        Snackbar.LENGTH_LONG).show();
                
                // Continue with a slight delay to allow user to read the message
                new Handler(Looper.getMainLooper()).postDelayed(() -> {
                    configureOpeningSequence();
                }, 2000);
            }
        });
    }

    /**
     * Configure and start the opening sound sequence
     */
    private void configureOpeningSequence() {
        try {
            permissionChecksComplete = true;
            
            // Create and configure media player
            mediaPlayer = MediaPlayer.create(this, R.raw.abertura2);
            
            // Set completion listener to navigate to next screen
            mediaPlayer.setOnCompletionListener(mp -> navigateToNextScreen());
            
            // Start playing the opening sound
            mediaPlayer.start();
            
            // Set a timeout in case the media player fails
            new Handler(Looper.getMainLooper()).postDelayed(() -> {
                if (!isFinishing() && !isDestroyed()) {
                    navigateToNextScreen();
                }
            }, SPLASH_TIMEOUT);
            
        } catch (Exception e) {
            Log.e(TAG, "Error configuring opening sequence", e);
            
            // Navigate even if there's an error
            new Handler(Looper.getMainLooper()).postDelayed(() -> {
                navigateToNextScreen();
            }, 2000);
        }
    }

    /**
     * Navigate to the appropriate next screen based on session state
     */
    private void navigateToNextScreen() {
        // Skip if already navigated
        if (isFinishing() || isDestroyed()) {
            return;
        }
        
        try {
            Intent intent;
            
            // Check if user is already logged in
            if (ControladoraFachadaSingleton.getInstance().temSessao()) {
                // Use the new OpenStreetMap MapActivity
                intent = new Intent(this, MapActivity.class);
            } else {
                intent = new Intent(this, LoginActivity.class);
            }
            
            startActivity(intent);
            finish();
            
        } catch (Exception e) {
            Log.e(TAG, "Error navigating to next screen", e);
            // Fallback to login activity if there's an error
            startActivity(new Intent(this, LoginActivity.class));
            finish();
        }
    }

    /**
     * Release media player resources
     */
    private void releaseMediaPlayer() {
        if (mediaPlayer != null) {
            try {
                if (mediaPlayer.isPlaying()) {
                    mediaPlayer.stop();
                }
                mediaPlayer.release();
                mediaPlayer = null;
            } catch (Exception e) {
                Log.e(TAG, "Error releasing media player", e);
            }
        }
    }
}
