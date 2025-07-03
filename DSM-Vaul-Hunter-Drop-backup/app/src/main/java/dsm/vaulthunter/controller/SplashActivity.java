package dsm.vaulthunter.controller;

import android.content.Intent;
import android.os.Bundle;
import android.os.Handler;
import android.view.View;
import android.view.animation.Animation;
import android.view.animation.AnimationUtils;
import android.widget.ImageView;

import androidx.appcompat.app.AppCompatActivity;

import dsm.vaulthunter.util.CutsceneManager;
import teste.lucasvegi.pokemongooffline.R;

/**
 * Splash screen activity for the Vault Hunter game
 */
public class SplashActivity extends AppCompatActivity {
    private static final String TAG = "SplashActivity";
    
    // Duration of splash screen in milliseconds
    private static final long SPLASH_DURATION = 2000;
    
    // Flag to track if cutscene is playing
    private boolean isCutscenePlaying = false;
    
    // Handler for delayed navigation
    private Handler handler;
    private Runnable navigateRunnable;
    
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        // Set theme to DSM theme
        setTheme(R.style.DSMTheme);
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_splash);
        
        // Initialize views
        ImageView logoImage = findViewById(R.id.splashLogo);
        
        // Apply logo animation
        Animation fadeIn = AnimationUtils.loadAnimation(this, android.R.anim.fade_in);
        fadeIn.setDuration(1500);
        logoImage.startAnimation(fadeIn);
        
        // Initialize handler and runnable for delayed navigation
        handler = new Handler();
        navigateRunnable = new Runnable() {
            @Override
            public void run() {
                navigateToNextScreen();
            }
        };
        
        // Try to play intro cutscene
        isCutscenePlaying = playCutsceneIfNeeded();
        
        // If cutscene is not playing, schedule navigation after splash
        if (!isCutscenePlaying) {
            handler.postDelayed(navigateRunnable, SPLASH_DURATION);
        }
    }
    
    /**
     * Play intro cutscene if it hasn't been viewed yet
     * @return true if cutscene is playing
     */
    private boolean playCutsceneIfNeeded() {
        CutsceneManager cutsceneManager = CutsceneManager.getInstance(this);
        
        // Check if intro cutscene exists and hasn't been viewed
        if (cutsceneManager.shouldPlayCutscene(CutsceneManager.CUTSCENE_INTRO) && 
            cutsceneManager.cutsceneExists(CutsceneManager.CUTSCENE_INTRO)) {
            
            // Schedule cutscene after splash delay
            handler.postDelayed(new Runnable() {
                @Override
                public void run() {
                    // Play cutscene
                    cutsceneManager.playCutscene(CutsceneManager.CUTSCENE_INTRO);
                }
            }, SPLASH_DURATION);
            
            return true;
        }
        
        return false;
    }
    
    /**
     * Navigate to the next screen (login)
     */
    private void navigateToNextScreen() {
        // Start LoginActivity
        Intent intent = new Intent(SplashActivity.this, LoginActivity.class);
        startActivity(intent);
        
        // Close splash activity
        finish();
    }
    
    @Override
    protected void onResume() {
        super.onResume();
        
        // If returning from cutscene, navigate to next screen
        if (isCutscenePlaying) {
            navigateToNextScreen();
        }
    }
    
    @Override
    protected void onDestroy() {
        super.onDestroy();
        
        // Remove pending callbacks to prevent leaks
        if (handler != null && navigateRunnable != null) {
            handler.removeCallbacks(navigateRunnable);
        }
    }
    
    @Override
    public void onWindowFocusChanged(boolean hasFocus) {
        super.onWindowFocusChanged(hasFocus);
        if (hasFocus) {
            hideSystemUI();
        }
    }
    
    /**
     * Hide system UI for immersive splash experience
     */
    private void hideSystemUI() {
        View decorView = getWindow().getDecorView();
        decorView.setSystemUiVisibility(
                View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY
                | View.SYSTEM_UI_FLAG_LAYOUT_STABLE
                | View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION
                | View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN
                | View.SYSTEM_UI_FLAG_HIDE_NAVIGATION
                | View.SYSTEM_UI_FLAG_FULLSCREEN);
    }
}