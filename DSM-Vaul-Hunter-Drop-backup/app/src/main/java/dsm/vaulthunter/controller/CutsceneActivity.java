package dsm.vaulthunter.controller;

import android.content.res.AssetFileDescriptor;
import android.media.MediaPlayer;
import android.net.Uri;
import android.os.Bundle;
import android.os.Handler;
import android.util.Log;
import android.view.View;
import android.widget.Button;
import android.widget.ProgressBar;
import android.widget.TextView;
import android.widget.VideoView;

import androidx.appcompat.app.AppCompatActivity;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;

import dsm.vaulthunter.util.CutsceneManager;
import teste.lucasvegi.pokemongooffline.R;

/**
 * Activity for playing cutscene videos
 */
public class CutsceneActivity extends AppCompatActivity {
    private static final String TAG = "CutsceneActivity";
    
    public static final String EXTRA_CUTSCENE_ID = "cutscene_id";
    public static final String EXTRA_CUTSCENE_FILENAME = "cutscene_filename";
    public static final String EXTRA_CUTSCENE_DESCRIPTION = "cutscene_description";
    public static final String EXTRA_CUTSCENE_SKIPPABLE = "cutscene_skippable";
    
    private String cutsceneId;
    private String filename;
    private String description;
    private boolean skippable;
    
    private VideoView videoView;
    private ProgressBar loadingIndicator;
    private Button skipButton;
    private TextView descriptionText;
    private TextView errorMessage;
    private Button continueButton;
    
    private Handler handler;
    private Runnable hideDescriptionRunnable;
    
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_cutscene);
        
        // Get cutscene info from intent
        cutsceneId = getIntent().getStringExtra(EXTRA_CUTSCENE_ID);
        filename = getIntent().getStringExtra(EXTRA_CUTSCENE_FILENAME);
        description = getIntent().getStringExtra(EXTRA_CUTSCENE_DESCRIPTION);
        skippable = getIntent().getBooleanExtra(EXTRA_CUTSCENE_SKIPPABLE, true);
        
        // Initialize views
        videoView = findViewById(R.id.cutsceneVideoView);
        loadingIndicator = findViewById(R.id.loadingIndicator);
        skipButton = findViewById(R.id.skipButton);
        descriptionText = findViewById(R.id.cutsceneDescription);
        errorMessage = findViewById(R.id.errorMessage);
        continueButton = findViewById(R.id.continueButton);
        
        // Set skip button visibility based on skippable flag
        skipButton.setVisibility(skippable ? View.VISIBLE : View.GONE);
        
        // Set cutscene description
        descriptionText.setText(description);
        
        // Initialize handler for hiding description
        handler = new Handler();
        hideDescriptionRunnable = new Runnable() {
            @Override
            public void run() {
                descriptionText.setVisibility(View.GONE);
            }
        };
        
        // Show description for 3 seconds then hide it
        handler.postDelayed(hideDescriptionRunnable, 3000);
        
        // Attempt to play the cutscene
        playCutscene();
    }
    
    /**
     * Play the cutscene video
     */
    private void playCutscene() {
        try {
            // Show loading indicator
            loadingIndicator.setVisibility(View.VISIBLE);
            
            // Get the cutscene file from assets
            Uri videoUri = getUriForAsset(filename);
            
            if (videoUri != null) {
                // Set up the video view
                videoView.setVideoURI(videoUri);
                videoView.setOnPreparedListener(new MediaPlayer.OnPreparedListener() {
                    @Override
                    public void onPrepared(MediaPlayer mp) {
                        // Hide loading indicator
                        loadingIndicator.setVisibility(View.GONE);
                        
                        // Start playback
                        videoView.start();
                    }
                });
                
                videoView.setOnCompletionListener(new MediaPlayer.OnCompletionListener() {
                    @Override
                    public void onCompletion(MediaPlayer mp) {
                        // Finish the activity when the video is done
                        finish();
                    }
                });
                
                videoView.setOnErrorListener(new MediaPlayer.OnErrorListener() {
                    @Override
                    public boolean onError(MediaPlayer mp, int what, int extra) {
                        // Show error message
                        handlePlaybackError("Error playing video: " + what + ", " + extra);
                        return true;
                    }
                });
            } else {
                // Show error message
                handlePlaybackError("Cutscene file not found: " + filename);
            }
        } catch (Exception e) {
            // Show error message
            handlePlaybackError("Error playing cutscene: " + e.getMessage());
            Log.e(TAG, "Error playing cutscene", e);
        }
    }
    
    /**
     * Get a URI for an asset file
     * @param assetFilename Asset filename
     * @return URI for the asset
     */
    private Uri getUriForAsset(String assetFilename) {
        try {
            // Create a temp file to copy the asset to
            File tempFile = new File(getCacheDir(), assetFilename);
            
            // Delete existing file if it exists
            if (tempFile.exists()) {
                tempFile.delete();
            }
            
            // Copy the asset to the temp file
            copyAssetToFile("videos/" + assetFilename, tempFile);
            
            // Return a URI for the temp file
            return Uri.fromFile(tempFile);
        } catch (IOException e) {
            Log.e(TAG, "Error getting URI for asset", e);
            return null;
        }
    }
    
    /**
     * Copy an asset to a file
     * @param assetPath Asset path
     * @param outFile Output file
     * @throws IOException If there's an error copying the asset
     */
    private void copyAssetToFile(String assetPath, File outFile) throws IOException {
        // Open the asset
        AssetFileDescriptor afd = getAssets().openFd(assetPath);
        
        // Create output stream
        FileOutputStream out = new FileOutputStream(outFile);
        
        // Copy the asset to the output stream
        InputStream in = afd.createInputStream();
        byte[] buffer = new byte[1024];
        int read;
        while ((read = in.read(buffer)) != -1) {
            out.write(buffer, 0, read);
        }
        
        // Close streams
        in.close();
        out.close();
        afd.close();
    }
    
    /**
     * Handle playback error
     * @param message Error message
     */
    private void handlePlaybackError(String message) {
        // Hide video and loading indicator
        videoView.setVisibility(View.GONE);
        loadingIndicator.setVisibility(View.GONE);
        
        // Show error message and continue button
        errorMessage.setText(message);
        errorMessage.setVisibility(View.VISIBLE);
        continueButton.setVisibility(View.VISIBLE);
        
        // Hide description and skip button
        descriptionText.setVisibility(View.GONE);
        skipButton.setVisibility(View.GONE);
        
        // Remove any pending callbacks
        handler.removeCallbacks(hideDescriptionRunnable);
        
        // Log the error
        Log.e(TAG, message);
    }
    
    /**
     * Skip the cutscene
     * @param view The view that was clicked
     */
    public void onSkipClick(View view) {
        // Mark cutscene as viewed in case it wasn't already
        CutsceneManager.getInstance(this).markCutsceneAsViewed(cutsceneId);
        
        // Stop video playback
        videoView.stopPlayback();
        
        // Finish the activity
        finish();
    }
    
    /**
     * Continue after an error
     * @param view The view that was clicked
     */
    public void onContinueClick(View view) {
        // Mark cutscene as viewed in case it wasn't already
        CutsceneManager.getInstance(this).markCutsceneAsViewed(cutsceneId);
        
        // Finish the activity
        finish();
    }
    
    @Override
    protected void onPause() {
        super.onPause();
        
        // Pause video playback
        if (videoView.isPlaying()) {
            videoView.pause();
        }
    }
    
    @Override
    protected void onResume() {
        super.onResume();
        
        // Resume video playback
        if (!videoView.isPlaying() && videoView.getVisibility() == View.VISIBLE) {
            videoView.resume();
        }
    }
    
    @Override
    protected void onDestroy() {
        super.onDestroy();
        
        // Clean up resources
        if (videoView != null) {
            videoView.stopPlayback();
        }
        
        // Remove any pending callbacks
        if (handler != null && hideDescriptionRunnable != null) {
            handler.removeCallbacks(hideDescriptionRunnable);
        }
    }
    
    @Override
    public void onBackPressed() {
        // Only allow back button if cutscene is skippable
        if (skippable) {
            onSkipClick(null);
        }
        // Otherwise ignore back button
    }
}