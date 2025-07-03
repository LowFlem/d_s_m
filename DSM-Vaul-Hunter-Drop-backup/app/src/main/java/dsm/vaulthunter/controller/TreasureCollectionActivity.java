package dsm.vaulthunter.controller;

import static dsm.vaulthunter.util.PermissionHelper.*;

import android.annotation.SuppressLint;
import android.content.Intent;
import android.content.pm.ActivityInfo;
import android.graphics.Point;
import android.hardware.Sensor;
import android.hardware.SensorEvent;
import android.hardware.SensorEventListener;
import android.hardware.SensorManager;
import android.media.MediaPlayer;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.util.Log;
import android.view.Display;
import android.view.MotionEvent;
import android.view.View;
import android.view.Window;
import android.view.WindowManager;
import android.widget.ImageView;
import android.widget.TextView;

import androidx.appcompat.app.AppCompatActivity;
import androidx.camera.core.CameraSelector;
import androidx.camera.core.Preview;
import androidx.camera.lifecycle.ProcessCameraProvider;
import androidx.camera.view.PreviewView;
import androidx.constraintlayout.widget.ConstraintLayout;
import androidx.core.content.ContextCompat;

import com.google.android.material.snackbar.Snackbar;
import com.google.common.util.concurrent.ListenableFuture;

import java.util.HashMap;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

import dsm.vaulthunter.model.AppController;
import dsm.vaulthunter.model.CollectedTreasure;
import dsm.vaulthunter.model.Treasure;
import dsm.vaulthunter.model.TreasureAppearance;
import dsm.vaulthunter.util.PermissionHelper;
import dsm.vaulthunter.util.ViewUnitsUtil;
import teste.lucasvegi.pokemongooffline.R;

/**
 * Activity for AR Treasure collection gameplay
 */
public class TreasureCollectionActivity extends AppCompatActivity implements SensorEventListener {
    private static final String TAG = "VH_TreasureCollection";
    
    // Sensors
    private SensorManager sensorManager;
    private Sensor gyroscope;
    private Sensor accelerometer;
    
    // UI Elements
    private ImageView treasureImageView;
    private ImageView collectorImageView;
    private TextView treasureNameTextView;
    private TextView statusLabelTextView;
    private PreviewView cameraPreviewView;
    
    // Screen dimensions
    private int screenWidth;
    private int screenHeight;
    private float centerX;
    private float centerY;
    private float scaleX;
    private float scaleY;
    private float centerXCollector;
    private float centerYCollector;
    
    // Gyroscope rotation tracking
    private float totalRotationX = 0;
    private float totalRotationY = 0;
    private float totalRotationZ = 0;
    private float prevRotationX = 0;
    private float prevRotationY = 0;
    private float prevRotationZ = 0;
    
    // Boundaries for treasure and screen
    private float distanceToTop;
    private float distanceToBottom;
    private float distanceToLeft;
    private float distanceToRight;
    
    // Treasure image sizing
    private float treasureImageScale = 0.5f;
    private boolean isTreasureImageReady = false;
    private float treasureImageWidth = 0;
    private float treasureImageHeight = 0;
    private float[] treasureBoundaries;
    
    // Collector image sizing
    private final float collectorImageScale = 0.15f;
    private boolean isCollectorImageReady = false;
    private float collectorImageWidth = 0;
    private float collectorImageHeight = 0;

    // Collection state
    private boolean isCollected = false;
    
    // Touch tracking for collector tool throw
    private float touchStartX = 0;
    private float touchStartY = 0;
    private float touchEndX = 0;
    private float touchEndY = 0;
    private long touchStartTime = 0;
    private long touchEndTime = 0;
    private float movementX;
    private float movementY;
    private long touchDuration;
    private float velocityX;
    private float velocityY;
    private float originalVelocityX;
    private float originalVelocityY;
    
    // Audio
    private MediaPlayer huntMusic;
    private MediaPlayer throwSound;
    private MediaPlayer bounceSound;
    private MediaPlayer collectionSuccessSound;
    private int bounceCount = 0;
    
    // Treasure data
    private Treasure treasure;
    private TreasureAppearance appearance;
    
    // Camera processing
    private ListenableFuture<ProcessCameraProvider> cameraProviderFuture;
    private ExecutorService cameraExecutor;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        
        // Setup fullscreen
        requestWindowFeature(Window.FEATURE_NO_TITLE);
        getWindow().addFlags(WindowManager.LayoutParams.FLAG_FULLSCREEN);
        
        setContentView(R.layout.activity_captura);
        setRequestedOrientation(ActivityInfo.SCREEN_ORIENTATION_UNSPECIFIED);
        
        // Initialize camera processing executor
        cameraExecutor = Executors.newSingleThreadExecutor();
        
        // Initialize sensor manager and sensors
        sensorManager = (SensorManager) getSystemService(SENSOR_SERVICE);
        gyroscope = sensorManager.getDefaultSensor(Sensor.TYPE_GYROSCOPE);
        accelerometer = sensorManager.getDefaultSensor(Sensor.TYPE_LINEAR_ACCELERATION);
        
        // Get screen dimensions
        Display display = getWindowManager().getDefaultDisplay();
        Point size = new Point();
        display.getSize(size);
        screenWidth = size.x;
        screenHeight = size.y;
        
        // Initialize UI elements
        initializeViews();
        
        // Check camera permission and initialize camera
        checkCameraPermissionAndInitialize();
        
        // Start hunt music
        initializeHuntMusic();
    }
    
    private void initializeViews() {
        treasureImageView = findViewById(R.id.pokemon);
        collectorImageView = findViewById(R.id.pokeball);
        treasureNameTextView = findViewById(R.id.txtNomePkmnCaptura);
        statusLabelTextView = findViewById(R.id.labelPkmnNovo);
        cameraPreviewView = findViewById(R.id.camera_preview);
    }
    
    private void checkCameraPermissionAndInitialize() {
        checkCameraPermission(this, new PermissionCallback() {
            @Override
            public void onPermissionsGranted() {
                initializeCamera();
            }

            @Override
            public void onPermissionsDenied() {
                Snackbar.make(findViewById(android.R.id.content),
                        R.string.camera_permission_rationale,
                        Snackbar.LENGTH_LONG).show();
                finish();
            }
        });
    }
    
    private void initializeCamera() {
        cameraProviderFuture = ProcessCameraProvider.getInstance(this);
        cameraProviderFuture.addListener(() -> {
            try {
                ProcessCameraProvider cameraProvider = cameraProviderFuture.get();
                
                // Set up the preview use case
                Preview preview = new Preview.Builder().build();
                
                // Choose back camera
                CameraSelector cameraSelector = new CameraSelector.Builder()
                        .requireLensFacing(CameraSelector.LENS_FACING_BACK)
                        .build();
                
                // Connect preview to the preview view
                preview.setSurfaceProvider(cameraPreviewView.getSurfaceProvider());
                
                // Unbind any bound use cases before rebinding
                cameraProvider.unbindAll();
                
                // Bind the camera to the lifecycle
                cameraProvider.bindToLifecycle(this, cameraSelector, preview);
                
            } catch (Exception e) {
                Log.e(TAG, "Use case binding failed", e);
            }
        }, ContextCompat.getMainExecutor(this));
    }
    
    private void initializeHuntMusic() {
        huntMusic = MediaPlayer.create(getBaseContext(), R.raw.battle);
        huntMusic.setLooping(true);
        adjustMediaPlayerVolume(huntMusic, 90); // Lower volume to compensate for loud audio
        huntMusic.start();
    }

    @Override
    protected void onResume() {
        super.onResume();
        
        // Get Treasure data from intent
        Intent intent = getIntent();
        appearance = (TreasureAppearance) intent.getSerializableExtra("treasure_appearance");
        assert appearance != null;
        treasure = appearance.getTreasure();
        
        // Set Treasure status label (new or known)
        if (AppController.getInstance().getLoggedInUser().getTreasureCount(treasure) > 0) {
            statusLabelTextView.setText("Known treasure");
        } else {
            statusLabelTextView.setText("New discovery!");
        }
        
        // Set Treasure name label
        treasureNameTextView.post(() -> {
            treasureNameTextView.setText(treasure.getName());
            treasureNameTextView.measure(0, 0);
            // Position the Treasure name at the top right with 8dp margin
            treasureNameTextView.setX(screenWidth - treasureNameTextView.getMeasuredWidth() - ViewUnitsUtil.convertDpToPixel(8));
        });
        
        // Set Treasure image - use a placeholder for now
        // In the actual implementation, you would use the proper resource based on treasure.getNumber()
        int treasureImageResourceId = getResources().getIdentifier(
                "p" + treasure.getNumber(), "drawable", getPackageName());
        
        if (treasureImageResourceId != 0) {
            treasureImageView.setImageResource(treasureImageResourceId);
        } else {
            // Fallback to a default image
            treasureImageView.setImageResource(R.drawable.p1);
        }
        
        // Initialize UI elements positioning
        configureCollector();
        configureTreasure();
        
        // Register sensor listeners
        if (gyroscope != null) {
            sensorManager.registerListener(this, gyroscope, SensorManager.SENSOR_DELAY_GAME);
        }
        
        if (accelerometer != null) {
            sensorManager.registerListener(this, accelerometer, SensorManager.SENSOR_DELAY_GAME);
        }
    }

    @Override
    protected void onPause() {
        super.onPause();
        
        // Unregister sensor listeners
        sensorManager.unregisterListener(this);
        isTreasureImageReady = false;
        isCollectorImageReady = false;
        
        // Pause hunt music
        if (huntMusic != null && huntMusic.isPlaying()) {
            huntMusic.pause();
        }
    }

    @Override
    protected void onRestart() {
        super.onRestart();
        
        // Resume hunt music
        if (huntMusic != null && !huntMusic.isPlaying()) {
            huntMusic.start();
        }
    }

    @Override
    protected void onDestroy() {
        // Release media resources
        releaseMediaPlayers();
        
        // Shutdown executor
        if (cameraExecutor != null && !cameraExecutor.isShutdown()) {
            cameraExecutor.shutdown();
        }
        
        super.onDestroy();
    }
    
    private void releaseMediaPlayers() {
        // Release hunt music
        if (huntMusic != null) {
            if (huntMusic.isPlaying()) {
                huntMusic.stop();
            }
            huntMusic.release();
            huntMusic = null;
        }
        
        // Release other sound effects
        releaseSoundEffect(throwSound);
        releaseSoundEffect(bounceSound);
        releaseSoundEffect(collectionSuccessSound);
    }
    
    private void releaseSoundEffect(MediaPlayer mediaPlayer) {
        if (mediaPlayer != null) {
            if (mediaPlayer.isPlaying()) {
                mediaPlayer.stop();
            }
            mediaPlayer.release();
        }
    }

    private void configureTreasure() {
        // Configure Treasure image sizing and position after view is laid out
        treasureImageView.post(() -> {
            // Set Treasure width based on screen size
            treasureImageWidth = screenWidth * treasureImageScale;
            
            // Calculate proportional height
            float proportion = (treasureImageWidth * 100) / treasureImageView.getMeasuredWidth();
            treasureImageHeight = treasureImageView.getMeasuredHeight() * proportion / 100;
            
            // Center the Treasure on screen
            centerX = (float) screenWidth / 2 - ((float) ((int) treasureImageWidth) / 2);
            centerY = (float) screenHeight / 2 - ((float) ((int) treasureImageHeight) / 2);
            
            // Update layout parameters
            ConstraintLayout.LayoutParams params = new ConstraintLayout.LayoutParams(
                    (int) treasureImageWidth, (int) treasureImageHeight);
            params.leftToLeft = ConstraintLayout.LayoutParams.PARENT_ID;
            params.topToTop = ConstraintLayout.LayoutParams.PARENT_ID;
            params.leftMargin = (int) centerX;
            params.topMargin = (int) centerY;
            treasureImageView.setLayoutParams(params);
            
            // Calculate distances for rotation handling
            distanceToTop = centerY;
            distanceToBottom = screenHeight - centerY;
            distanceToLeft = centerX;
            distanceToRight = screenWidth - centerX;
            
            // Calculate scale factors for gyroscope
            scaleX = (float) screenWidth / 72; // Each degree is worth scale pixels - 72° is the field of view
            scaleY = (float) screenHeight / 72;
            
            Log.i(TAG, "Dimensions: W: " + screenWidth + " H: " + screenHeight + 
                       " CX: " + centerX + " CY: " + centerY +
                       " TreasureW: " + (int) treasureImageWidth + " TreasureH: " + (int) treasureImageHeight);
            
            isTreasureImageReady = true;
            
            // Calculate initial Treasure boundaries
            treasureBoundaries = getImageBoundaries(
                    treasureImageView.getX(), treasureImageView.getY(), 
                    treasureImageHeight, treasureImageWidth);
        });
    }

    private void configureCollector() {
        // Configure Collector image sizing and position after view is laid out
        collectorImageView.post(() -> {
            // Set Collector width based on screen size
            collectorImageWidth = screenWidth * collectorImageScale;
            
            // Calculate proportional height
            float proportion = (collectorImageWidth * 100) / collectorImageView.getMeasuredWidth();
            collectorImageHeight = collectorImageView.getMeasuredHeight() * proportion / 100;
            
            // Position Collector at bottom center
            centerXCollector = (float) screenWidth / 2 - ((float) ((int) collectorImageWidth) / 2);
            centerYCollector = screenHeight - (int) collectorImageHeight - ViewUnitsUtil.convertDpToPixel(24);
            
            // Update layout parameters
            ConstraintLayout.LayoutParams params = new ConstraintLayout.LayoutParams(
                    (int) collectorImageWidth, (int) collectorImageHeight);
            params.leftToLeft = ConstraintLayout.LayoutParams.PARENT_ID;
            params.bottomToBottom = ConstraintLayout.LayoutParams.PARENT_ID;
            params.leftMargin = (int) centerXCollector;
            params.bottomMargin = (int) (screenHeight - centerYCollector - collectorImageHeight);
            collectorImageView.setLayoutParams(params);
            
            Log.i(TAG, "Collector: W: " + screenWidth + " H: " + screenHeight + 
                       " CX: " + centerXCollector + " CY: " + centerYCollector +
                       " CollectorW: " + (int) collectorImageWidth + " CollectorH: " + (int) collectorImageHeight);
            
            isCollectorImageReady = true;
            
            // Configure touch listener for throwing
            configureCollectorTouchListener();
        });
    }

    @SuppressLint("ClickableViewAccessibility")
    private void configureCollectorTouchListener() {
        // Set touch listener for Collector throwing
        collectorImageView.setOnTouchListener((v, event) -> {
            float x = event.getRawX();
            float y = event.getRawY();
            
            switch (event.getAction() & MotionEvent.ACTION_MASK) {
                case MotionEvent.ACTION_DOWN:
                    // Play throw sound
                    playThrowSound();
                    
                    // Record initial touch position and time
                    touchStartTime = System.currentTimeMillis();
                    touchStartX = collectorImageView.getX();
                    touchStartY = collectorImageView.getY();
                    return true;
                    
                case MotionEvent.ACTION_UP:
                    // User released finger - calculate throw velocity
                    touchEndTime = System.currentTimeMillis();
                    touchEndX = collectorImageView.getX();
                    touchEndY = collectorImageView.getY();
                    
                    // Calculate distance and time
                    movementX = Math.abs(touchStartX - touchEndX);
                    movementY = Math.abs(touchStartY - touchEndY);
                    touchDuration = touchEndTime - touchStartTime;
                    
                    // Calculate velocity (pixels/ms)
                    velocityX = movementX / touchDuration;
                    velocityY = movementY / touchDuration;
                    
                    // Store original values for deceleration
                    originalVelocityX = velocityX;
                    originalVelocityY = velocityY;
                    
                    // Process the Collector throw
                    processCollectorThrow();
                    return true;
                    
                case MotionEvent.ACTION_MOVE:
                    // Move Collector with touch
                    collectorImageView.setX(x - (collectorImageWidth / 2));
                    collectorImageView.setY((y - (collectorImageHeight / 3)) - (collectorImageHeight / 2));
                    
                    // Check for collection during movement
                    checkForCollection();
                    return true;
                    
                default:
                    return false;
            }
        });
    }
    
    /** @noinspection BusyWait*/
    private void processCollectorThrow() {
        // Create a thread to animate the Collector throw
        new Thread(() -> {
            // Continue animating while Collector has momentum and hasn't collected
            while (velocityX > 0 && velocityY > 0 && !isCollected) {
                // Acceleration factor
                final int timeStep = 25;
                
                // Update Collector position on UI thread
                runOnUiThread(() -> {
                    // Move horizontally
                    if (touchEndX >= touchStartX) {
                        collectorImageView.setX(collectorImageView.getX() + (timeStep * velocityX));
                    } else {
                        collectorImageView.setX(collectorImageView.getX() - (timeStep * velocityX));
                    }
                    
                    // Move vertically
                    if (touchEndY <= touchStartY) {
                        collectorImageView.setY(collectorImageView.getY() - (timeStep * velocityY));
                    } else {
                        collectorImageView.setY(collectorImageView.getY() + (timeStep * velocityY));
                    }
                    
                    // Check for collection during throw
                    checkForCollection();
                    
                    // Reduce velocity over time
                    velocityX = velocityX - (originalVelocityX * 0.045f);
                    velocityY = velocityY - (originalVelocityY * 0.045f);
                });
                
                // Small delay for animation
                try {
                    Thread.sleep(timeStep);
                } catch (InterruptedException e) {
                    Log.e(TAG, "Thread interrupted during throw animation", e);
                    Thread.currentThread().interrupt();
                    return;
                }
            }
            
            // Check if Collector went off-screen
            runOnUiThread(() -> {
                if (collectorImageView.getX() > screenWidth || collectorImageView.getX() < 0 ||
                    collectorImageView.getY() > screenHeight || collectorImageView.getY() < 0) {
                    
                    Snackbar.make(findViewById(android.R.id.content), 
                            "Missed! Try again.", Snackbar.LENGTH_SHORT).show();
                    
                    // Reset Collector position
                    collectorImageView.setX(centerXCollector);
                    collectorImageView.setY(centerYCollector);
                }
            });
        }).start();
    }
    
    private void checkForCollection() {
        // Get Collector boundaries
        float[] collectorBoundaries = getImageBoundaries(
                collectorImageView.getX(), collectorImageView.getY(),
                collectorImageHeight, collectorImageWidth);
        
        // Check for intersection with Treasure
        if (isIntersecting(collectorBoundaries, treasureBoundaries) && !isCollected) {
            isCollected = true;
            configureCollectionEffect();
        }
    }
    
    private void configureCollectionEffect() {
        // Play bounce sound
        bounceSound = MediaPlayer.create(getBaseContext(), R.raw.quicando);
        bounceSound.setOnCompletionListener(mediaPlayer -> {
            int maxBounces = 3;
            if (bounceCount < maxBounces) {
                bounceCount++;
                mediaPlayer.seekTo(0);
                mediaPlayer.start();
                
                // Animate Collector with bounce sound
                if (bounceCount % 2 == 0) {
                    collectorImageView.animate().rotation(0).start();
                } else {
                    collectorImageView.animate().rotation(-20).start();
                }
            } else {
                bounceCount = 0;
                
                // Play success sound
                collectionSuccessSound = MediaPlayer.create(getBaseContext(), R.raw.sucesso);
                collectionSuccessSound.setOnCompletionListener(mp -> {
                    // Add Treasure to user's collection
                    AppController.getInstance().getLoggedInUser().collectTreasure(appearance);
                    AppController.getInstance().addExperiencePoints("treasure");
                    
                    // Close the collection activity
                    finish();
                });
                
                // Pause hunt music and play success
                huntMusic.pause();
                collectionSuccessSound.start();
                
                // Show success message
                Snackbar.make(findViewById(android.R.id.content),
                        "You collected " + treasure.getName() + "!",
                        Snackbar.LENGTH_LONG).show();
            }
        });
        
        // Lower hunt music volume before Collector bounces
        adjustMediaPlayerVolume(huntMusic, 85);
        
        // Start bounce sound
        bounceSound.start();
        
        // Replace Treasure image with sparkle/explosion effect
        treasureImageView.setImageResource(R.drawable.explosion);
        
        // Hide Treasure after explosion animation
        new Handler(Looper.getMainLooper()).postDelayed(() -> 
                treasureImageView.setVisibility(View.INVISIBLE), 350);
    }
    
    private void playThrowSound() {
        if (throwSound != null) {
            throwSound.release();
        }
        throwSound = MediaPlayer.create(this, R.raw.arremesso);
        throwSound.start();
    }
    
    private void adjustMediaPlayerVolume(MediaPlayer mediaPlayer, int volume) {
        int maxVolume = 100;
        float log = (float) (Math.log(maxVolume - volume) / Math.log(maxVolume));
        mediaPlayer.setVolume(1 - log, 1 - log);
    }
    
    private float[] getImageBoundaries(float x, float y, float height, float width) {
        float[] boundaries = new float[4];
        boundaries[0] = x;               // Left
        boundaries[1] = x + width;       // Right
        boundaries[2] = y;               // Top
        boundaries[3] = y + height;      // Bottom
        
        Log.d(TAG, "Boundaries: L: " + boundaries[0] + " R: " + boundaries[1] + 
                    " T: " + boundaries[2] + " B: " + boundaries[3]);
        return boundaries;
    }
    
    private boolean isIntersecting(float[] obj1, float[] obj2) {
        return obj1[0] <= obj2[1] &&    // obj1 left <= obj2 right
               obj2[0] <= obj1[1] &&    // obj2 left <= obj1 right
               obj1[2] <= obj2[3] &&    // obj1 top <= obj2 bottom
               obj2[2] <= obj1[3];      // obj2 top <= obj1 bottom
    }

    // Sensor event handling
    private final HashMap<Integer, Long> sensorTimestamps = new HashMap<>();
    
    private double getElapsedSeconds(SensorEvent event) {
        Long lastTimestamp = sensorTimestamps.put(event.sensor.getType(), event.timestamp);
        
        if (lastTimestamp == null)
            return 0;
            
        return (event.timestamp - lastTimestamp) / 1_000_000_000.0;
    }
    
    // Acceleration handling variables
    private double accelerationNoise, speed, distance;
    private long accelerationSamples;
    
    @Override
    public void onSensorChanged(SensorEvent event) {
        switch (event.sensor.getType()) {
            case Sensor.TYPE_GYROSCOPE:
                handleGyroscopeEvent(event);
                break;
            case Sensor.TYPE_LINEAR_ACCELERATION:
                handleAccelerationEvent(event);
                break;
        }
    }
    
    private void handleGyroscopeEvent(SensorEvent event) {
        if (!isTreasureImageReady || !isCollectorImageReady)
            return;
            
        // Keep screen on
        getWindow().addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);
        
        // Read gyroscope values
        float x = event.values[0];
        float y = event.values[1];
        float z = event.values[2];
        
        // Convert to degrees and factor in time
        float newRotationX = (float) ((x * 57.2958) * 0.02); // 0.02 seconds due to SENSOR_DELAY_GAME
        float newRotationY = (float) ((y * 57.2958) * 0.02);
        float newRotationZ = (float) ((z * 57.2958) * 0.02);
        
        // Update total rotation
        totalRotationX += newRotationX;
        totalRotationY += newRotationY;
        totalRotationZ += newRotationZ;
        
        // Update Treasure position based on rotation
        treasureImageView.getX();
        float newX;
        treasureImageView.getY();
        float newY;
        
        // Update X position if significant movement
        if (totalRotationY > prevRotationY + 0.01 || totalRotationY < prevRotationY - 0.01) {
            newX = treasureImageView.getX() + (newRotationY * scaleX);
            treasureImageView.setX(newX);
        } else {
            totalRotationY = prevRotationY; // Eliminate small sensor noise
        }
        
        // Update Y position if significant movement
        if (totalRotationX > prevRotationX + 0.01 || totalRotationX < prevRotationX - 0.01) {
            newY = treasureImageView.getY() + (newRotationX * scaleY);
            treasureImageView.setY(newY);
        } else {
            totalRotationX = prevRotationX; // Eliminate small sensor noise
        }
        
        // Update rotation if significant
        if (totalRotationZ > prevRotationZ + 0.01 || totalRotationZ < prevRotationZ - 0.01) {
            treasureImageView.setRotation(totalRotationZ);
        } else {
            totalRotationZ = prevRotationZ; // Eliminate small sensor noise
        }
        
        // Save current rotation for next comparison
        prevRotationX = totalRotationX;
        prevRotationY = totalRotationY;
        prevRotationZ = totalRotationZ;
        
        // Handle wraparound for 360-degree rotation
        handleRotationWraparound();
        
        // Update Treasure boundaries
        treasureBoundaries = getImageBoundaries(
                treasureImageView.getX(), treasureImageView.getY(), 
                treasureImageHeight, treasureImageWidth);
    }
    
    private void handleRotationWraparound() {
        // Handle horizontal wraparound to right
        if (totalRotationY < 0) {
            if (Math.abs(Math.abs(totalRotationY) - 360) <= distanceToRight / scaleX) {
                treasureImageView.setX(screenWidth - 10);
                centerX = treasureImageView.getX();
                distanceToLeft = centerX;
                distanceToRight = screenWidth - centerX;
                totalRotationY = 0;
            }
        }
        
        // Handle horizontal wraparound to left
        if (totalRotationY > 0) {
            if (Math.abs(Math.abs(totalRotationY) - 360) <= (distanceToLeft + treasureImageWidth) / scaleX) {
                treasureImageView.setX(-treasureImageWidth);
                centerX = treasureImageView.getX();
                distanceToLeft = scaleX - treasureImageWidth;
                distanceToRight = screenWidth - centerX;
                totalRotationY = 0;
            }
        }
        
        // Handle vertical wraparound to top
        if (totalRotationX > 0) {
            if (Math.abs(Math.abs(totalRotationX) - 360) <= (distanceToTop + treasureImageHeight) / scaleY) {
                treasureImageView.setY(-treasureImageHeight);
                centerY = treasureImageView.getY();
                distanceToTop = scaleY - treasureImageHeight;
                distanceToBottom = screenHeight - centerY;
                totalRotationX = 0;
            }
        }
        
        // Handle vertical wraparound to bottom
        if (totalRotationX < 0) {
            if (Math.abs(Math.abs(totalRotationX) - 360) <= distanceToBottom / scaleY) {
                treasureImageView.setY(screenHeight - 10);
                centerY = treasureImageView.getY();
                distanceToTop = centerY;
                distanceToBottom = screenHeight - centerY;
                totalRotationX = 0;
            }
        }
    }
    
    private void handleAccelerationEvent(SensorEvent event) {
        double elapsed = getElapsedSeconds(event);
        
        double accelerationSensor = -event.values[2];
        double accelerationMagnitude = Math.abs(accelerationSensor);
        double accelerationDirection = Math.signum(accelerationSensor);
        
        accelerationNoise += (1.0 / ++accelerationSamples) * (accelerationMagnitude - accelerationNoise);
        
        if (isTreasureImageReady && isCollectorImageReady) {
            // Reduce acceleration noise
            double acceleration = Math.max(accelerationMagnitude - accelerationNoise, 0) * accelerationDirection;
            
            // Update speed and distance
            speed = clamp(speed + acceleration * elapsed, -0.25, 0.25);
            distance = clamp(distance + speed * elapsed, 0.25, 0.75);
            
            // Update Treasure size based on distance
            treasureImageScale = 1.0f - (float) distance;
            configureTreasure();
        }
    }
    
    private double clamp(double value, double min, double max) {
        return Math.max(min, Math.min(max, value));
    }

    @Override
    public void onAccuracyChanged(Sensor sensor, int accuracy) {
        // Handle sensor accuracy changes if needed
    }
}