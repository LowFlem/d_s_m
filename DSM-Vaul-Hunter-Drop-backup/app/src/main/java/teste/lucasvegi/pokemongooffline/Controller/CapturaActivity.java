package teste.lucasvegi.pokemongooffline.Controller;

import static teste.lucasvegi.pokemongooffline.Util.PermissionHelper.*;

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

import teste.lucasvegi.pokemongooffline.Model.Aparecimento;
import teste.lucasvegi.pokemongooffline.Model.ControladoraFachadaSingleton;
import teste.lucasvegi.pokemongooffline.Model.Pokemon;
import teste.lucasvegi.pokemongooffline.R;
import teste.lucasvegi.pokemongooffline.Util.PermissionHelper;
import teste.lucasvegi.pokemongooffline.Util.ViewUnitsUtil;

/**
 * Activity for Pokemon capture gameplay
 */
public class CapturaActivity extends AppCompatActivity implements SensorEventListener {
    private static final String TAG = "CapturaActivity";
    
    // Sensors
    private SensorManager sensorManager;
    private Sensor gyroscope;
    private Sensor accelerometer;
    
    // UI Elements
    private ImageView pokemonImageView;
    private ImageView pokeballImageView;
    private TextView pokemonNameTextView;
    private TextView statusLabelTextView;
    private PreviewView cameraPreviewView;
    
    // Screen dimensions
    private int screenWidth;
    private int screenHeight;
    private float centerX;
    private float centerY;
    private float scaleX;
    private float scaleY;
    private float centerXPokeball;
    private float centerYPokeball;
    
    // Gyroscope rotation tracking
    private float totalRotationX = 0;
    private float totalRotationY = 0;
    private float totalRotationZ = 0;
    private float prevRotationX = 0;
    private float prevRotationY = 0;
    private float prevRotationZ = 0;
    
    // Boundaries for Pokemon and screen
    private float distanceToTop;
    private float distanceToBottom;
    private float distanceToLeft;
    private float distanceToRight;
    
    // Pokemon image sizing
    private float pokemonImageScale = 0.5f;
    private boolean isPokemonImageReady = false;
    private float pokemonImageWidth = 0;
    private float pokemonImageHeight = 0;
    private float[] pokemonBoundaries;
    
    // Pokeball image sizing
    private final float pokeballImageScale = 0.15f;
    private boolean isPokeballImageReady = false;
    private float pokeballImageWidth = 0;
    private float pokeballImageHeight = 0;

    // Capture state
    private boolean isCaptured = false;
    
    // Touch tracking for Pokeball throw
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
    private MediaPlayer battleMusic;
    private MediaPlayer throwSound;
    private MediaPlayer bounceSound;
    private MediaPlayer captureSuccessSound;
    private int bounceCount = 0;
    
    // Pokemon data
    private Pokemon pokemon;
    private Aparecimento appearance;
    
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
        
        // Start battle music
        initializeBattleMusic();
    }
    
    private void initializeViews() {
        pokemonImageView = findViewById(R.id.pokemon);
        pokeballImageView = findViewById(R.id.pokeball);
        pokemonNameTextView = findViewById(R.id.txtNomePkmnCaptura);
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
    
    private void initializeBattleMusic() {
        battleMusic = MediaPlayer.create(getBaseContext(), R.raw.battle);
        battleMusic.setLooping(true);
        adjustMediaPlayerVolume(battleMusic, 90); // Lower volume to compensate for loud audio
        battleMusic.start();
    }

    @Override
    protected void onResume() {
        super.onResume();
        
        // Get Pokemon data from intent
        Intent intent = getIntent();
        appearance = (Aparecimento) intent.getSerializableExtra("pkmn");
        assert appearance != null;
        pokemon = appearance.getPokemon();
        
        // Set Pokemon status label (new or known)
        if (ControladoraFachadaSingleton.getInstance().getUsuario().getQuantidadeCapturas(pokemon) > 0) {
            statusLabelTextView.setText(getString(R.string.capture_status_known));
        } else {
            statusLabelTextView.setText(getString(R.string.capture_status_new));
        }
        
        // Set Pokemon name label
        pokemonNameTextView.post(() -> {
            pokemonNameTextView.setText(pokemon.getNome());
            pokemonNameTextView.measure(0, 0);
            // Position the Pokemon name at the top right with 8dp margin
            pokemonNameTextView.setX(screenWidth - pokemonNameTextView.getMeasuredWidth() - ViewUnitsUtil.convertDpToPixel(8));
        });
        
        // Set Pokemon image
        pokemonImageView.setImageResource(pokemon.getFoto());
        
        // Initialize UI elements positioning
        configurePokeball();
        configurePokemon();
        
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
        isPokemonImageReady = false;
        isPokeballImageReady = false;
        
        // Pause battle music
        if (battleMusic != null && battleMusic.isPlaying()) {
            battleMusic.pause();
        }
    }

    @Override
    protected void onRestart() {
        super.onRestart();
        
        // Resume battle music
        if (battleMusic != null && !battleMusic.isPlaying()) {
            battleMusic.start();
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
        // Release battle music
        if (battleMusic != null) {
            if (battleMusic.isPlaying()) {
                battleMusic.stop();
            }
            battleMusic.release();
            battleMusic = null;
        }
        
        // Release other sound effects
        releaseSoundEffect(throwSound);
        releaseSoundEffect(bounceSound);
        releaseSoundEffect(captureSuccessSound);
    }
    
    private void releaseSoundEffect(MediaPlayer mediaPlayer) {
        if (mediaPlayer != null) {
            if (mediaPlayer.isPlaying()) {
                mediaPlayer.stop();
            }
            mediaPlayer.release();
        }
    }

    private void configurePokemon() {
        // Configure Pokemon image sizing and position after view is laid out
        pokemonImageView.post(() -> {
            // Set Pokemon width based on screen size
            pokemonImageWidth = screenWidth * pokemonImageScale;
            
            // Calculate proportional height
            float proportion = (pokemonImageWidth * 100) / pokemonImageView.getMeasuredWidth();
            pokemonImageHeight = pokemonImageView.getMeasuredHeight() * proportion / 100;
            
            // Center the Pokemon on screen
            centerX = (float) screenWidth / 2 - ((float) ((int) pokemonImageWidth) / 2);
            centerY = (float) screenHeight / 2 - ((float) ((int) pokemonImageHeight) / 2);
            
            // Update layout parameters
            ConstraintLayout.LayoutParams params = new ConstraintLayout.LayoutParams(
                    (int) pokemonImageWidth, (int) pokemonImageHeight);
            params.leftToLeft = ConstraintLayout.LayoutParams.PARENT_ID;
            params.topToTop = ConstraintLayout.LayoutParams.PARENT_ID;
            params.leftMargin = (int) centerX;
            params.topMargin = (int) centerY;
            pokemonImageView.setLayoutParams(params);
            
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
                       " PokemonW: " + (int) pokemonImageWidth + " PokemonH: " + (int) pokemonImageHeight);
            
            isPokemonImageReady = true;
            
            // Calculate initial Pokemon boundaries
            pokemonBoundaries = getImageBoundaries(
                    pokemonImageView.getX(), pokemonImageView.getY(), 
                    pokemonImageHeight, pokemonImageWidth);
        });
    }

    private void configurePokeball() {
        // Configure Pokeball image sizing and position after view is laid out
        pokeballImageView.post(() -> {
            // Set Pokeball width based on screen size
            pokeballImageWidth = screenWidth * pokeballImageScale;
            
            // Calculate proportional height
            float proportion = (pokeballImageWidth * 100) / pokeballImageView.getMeasuredWidth();
            pokeballImageHeight = pokeballImageView.getMeasuredHeight() * proportion / 100;
            
            // Position Pokeball at bottom center
            centerXPokeball = (float) screenWidth / 2 - ((float) ((int) pokeballImageWidth) / 2);
            centerYPokeball = screenHeight - (int) pokeballImageHeight - ViewUnitsUtil.convertDpToPixel(24);
            
            // Update layout parameters
            ConstraintLayout.LayoutParams params = new ConstraintLayout.LayoutParams(
                    (int) pokeballImageWidth, (int) pokeballImageHeight);
            params.leftToLeft = ConstraintLayout.LayoutParams.PARENT_ID;
            params.bottomToBottom = ConstraintLayout.LayoutParams.PARENT_ID;
            params.leftMargin = (int) centerXPokeball;
            params.bottomMargin = (int) (screenHeight - centerYPokeball - pokeballImageHeight);
            pokeballImageView.setLayoutParams(params);
            
            Log.i(TAG, "Pokeball: W: " + screenWidth + " H: " + screenHeight + 
                       " CX: " + centerXPokeball + " CY: " + centerYPokeball +
                       " BallW: " + (int) pokeballImageWidth + " BallH: " + (int) pokeballImageHeight);
            
            isPokeballImageReady = true;
            
            // Configure touch listener for throwing
            configurePokeballTouchListener();
        });
    }

    @SuppressLint("ClickableViewAccessibility")
    private void configurePokeballTouchListener() {
        // Set touch listener for Pokeball throwing
        pokeballImageView.setOnTouchListener((v, event) -> {
            float x = event.getRawX();
            float y = event.getRawY();
            
            switch (event.getAction() & MotionEvent.ACTION_MASK) {
                case MotionEvent.ACTION_DOWN:
                    // Play throw sound
                    playThrowSound();
                    
                    // Record initial touch position and time
                    touchStartTime = System.currentTimeMillis();
                    touchStartX = pokeballImageView.getX();
                    touchStartY = pokeballImageView.getY();
                    return true;
                    
                case MotionEvent.ACTION_UP:
                    // User released finger - calculate throw velocity
                    touchEndTime = System.currentTimeMillis();
                    touchEndX = pokeballImageView.getX();
                    touchEndY = pokeballImageView.getY();
                    
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
                    
                    // Process the Pokeball throw
                    processPokeballThrow();
                    return true;
                    
                case MotionEvent.ACTION_MOVE:
                    // Move Pokeball with touch
                    pokeballImageView.setX(x - (pokeballImageWidth / 2));
                    pokeballImageView.setY((y - (pokeballImageHeight / 3)) - (pokeballImageHeight / 2));
                    
                    // Check for capture during movement
                    checkForCapture();
                    return true;
                    
                default:
                    return false;
            }
        });
    }
    
    /** @noinspection BusyWait*/
    private void processPokeballThrow() {
        // Create a thread to animate the Pokeball throw
        new Thread(() -> {
            // Continue animating while Pokeball has momentum and hasn't captured
            while (velocityX > 0 && velocityY > 0 && !isCaptured) {
                // Acceleration factor
                final int timeStep = 25;
                
                // Update Pokeball position on UI thread
                runOnUiThread(() -> {
                    // Move horizontally
                    if (touchEndX >= touchStartX) {
                        pokeballImageView.setX(pokeballImageView.getX() + (timeStep * velocityX));
                    } else {
                        pokeballImageView.setX(pokeballImageView.getX() - (timeStep * velocityX));
                    }
                    
                    // Move vertically
                    if (touchEndY <= touchStartY) {
                        pokeballImageView.setY(pokeballImageView.getY() - (timeStep * velocityY));
                    } else {
                        pokeballImageView.setY(pokeballImageView.getY() + (timeStep * velocityY));
                    }
                    
                    // Check for capture during throw
                    checkForCapture();
                    
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
            
            // Check if Pokeball went off-screen
            runOnUiThread(() -> {
                if (pokeballImageView.getX() > screenWidth || pokeballImageView.getX() < 0 ||
                    pokeballImageView.getY() > screenHeight || pokeballImageView.getY() < 0) {
                    
                    Snackbar.make(findViewById(android.R.id.content), 
                            R.string.throw_again, Snackbar.LENGTH_SHORT).show();
                    
                    // Reset Pokeball position
                    pokeballImageView.setX(centerXPokeball);
                    pokeballImageView.setY(centerYPokeball);
                }
            });
        }).start();
    }
    
    private void checkForCapture() {
        // Get Pokeball boundaries
        float[] pokeballBoundaries = getImageBoundaries(
                pokeballImageView.getX(), pokeballImageView.getY(),
                pokeballImageHeight, pokeballImageWidth);
        
        // Check for intersection with Pokemon
        if (isIntersecting(pokeballBoundaries, pokemonBoundaries) && !isCaptured) {
            isCaptured = true;
            configureCaptureEffect();
        }
    }
    
    private void configureCaptureEffect() {
        // Play bounce sound
        bounceSound = MediaPlayer.create(getBaseContext(), R.raw.quicando);
        bounceSound.setOnCompletionListener(mediaPlayer -> {
            int maxBounces = 3;
            if (bounceCount < maxBounces) {
                bounceCount++;
                mediaPlayer.seekTo(0);
                mediaPlayer.start();
                
                // Animate Pokeball with bounce sound
                if (bounceCount % 2 == 0) {
                    pokeballImageView.animate().rotation(0).start();
                } else {
                    pokeballImageView.animate().rotation(-20).start();
                }
            } else {
                bounceCount = 0;
                
                // Play success sound
                captureSuccessSound = MediaPlayer.create(getBaseContext(), R.raw.sucesso);
                captureSuccessSound.setOnCompletionListener(mp -> {
                    // Add Pokemon to user's collection
                    ControladoraFachadaSingleton.getInstance().getUsuario().capturar(appearance);
                    ControladoraFachadaSingleton.getInstance().aumentaXp("captura");
                    
                    // Close the capture activity
                    finish();
                });
                
                // Pause battle music and play success
                battleMusic.pause();
                captureSuccessSound.start();
                
                // Show success message
                Snackbar.make(findViewById(android.R.id.content),
                        getString(R.string.capture_success, pokemon.getNome()),
                        Snackbar.LENGTH_LONG).show();
            }
        });
        
        // Lower battle music volume before Pokeball bounces
        adjustMediaPlayerVolume(battleMusic, 85);
        
        // Start bounce sound
        bounceSound.start();
        
        // Replace Pokemon image with explosion
        pokemonImageView.setImageResource(R.drawable.explosion);
        
        // Hide Pokemon after explosion animation
        new Handler(Looper.getMainLooper()).postDelayed(() -> 
                pokemonImageView.setVisibility(View.INVISIBLE), 350);
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
        if (!isPokemonImageReady || !isPokeballImageReady)
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
        
        // Update Pokemon position based on rotation
        pokemonImageView.getX();
        float newX;
        pokemonImageView.getY();
        float newY;
        
        // Update X position if significant movement
        if (totalRotationY > prevRotationY + 0.01 || totalRotationY < prevRotationY - 0.01) {
            newX = pokemonImageView.getX() + (newRotationY * scaleX);
            pokemonImageView.setX(newX);
        } else {
            totalRotationY = prevRotationY; // Eliminate small sensor noise
        }
        
        // Update Y position if significant movement
        if (totalRotationX > prevRotationX + 0.01 || totalRotationX < prevRotationX - 0.01) {
            newY = pokemonImageView.getY() + (newRotationX * scaleY);
            pokemonImageView.setY(newY);
        } else {
            totalRotationX = prevRotationX; // Eliminate small sensor noise
        }
        
        // Update rotation if significant
        if (totalRotationZ > prevRotationZ + 0.01 || totalRotationZ < prevRotationZ - 0.01) {
            pokemonImageView.setRotation(totalRotationZ);
        } else {
            totalRotationZ = prevRotationZ; // Eliminate small sensor noise
        }
        
        // Save current rotation for next comparison
        prevRotationX = totalRotationX;
        prevRotationY = totalRotationY;
        prevRotationZ = totalRotationZ;
        
        // Handle wraparound for 360-degree rotation
        handleRotationWraparound();
        
        // Update Pokemon boundaries
        pokemonBoundaries = getImageBoundaries(
                pokemonImageView.getX(), pokemonImageView.getY(), 
                pokemonImageHeight, pokemonImageWidth);
    }
    
    private void handleRotationWraparound() {
        // Handle horizontal wraparound to right
        if (totalRotationY < 0) {
            if (Math.abs(Math.abs(totalRotationY) - 360) <= distanceToRight / scaleX) {
                pokemonImageView.setX(screenWidth - 10);
                centerX = pokemonImageView.getX();
                distanceToLeft = centerX;
                distanceToRight = screenWidth - centerX;
                totalRotationY = 0;
            }
        }
        
        // Handle horizontal wraparound to left
        if (totalRotationY > 0) {
            if (Math.abs(Math.abs(totalRotationY) - 360) <= (distanceToLeft + pokemonImageWidth) / scaleX) {
                pokemonImageView.setX(-pokemonImageWidth);
                centerX = pokemonImageView.getX();
                distanceToLeft = scaleX - pokemonImageWidth;
                distanceToRight = screenWidth - centerX;
                totalRotationY = 0;
            }
        }
        
        // Handle vertical wraparound to top
        if (totalRotationX > 0) {
            if (Math.abs(Math.abs(totalRotationX) - 360) <= (distanceToTop + pokemonImageHeight) / scaleY) {
                pokemonImageView.setY(-pokemonImageHeight);
                centerY = pokemonImageView.getY();
                distanceToTop = scaleY - pokemonImageHeight;
                distanceToBottom = screenHeight - centerY;
                totalRotationX = 0;
            }
        }
        
        // Handle vertical wraparound to bottom
        if (totalRotationX < 0) {
            if (Math.abs(Math.abs(totalRotationX) - 360) <= distanceToBottom / scaleY) {
                pokemonImageView.setY(screenHeight - 10);
                centerY = pokemonImageView.getY();
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
        
        if (isPokemonImageReady && isPokeballImageReady) {
            // Reduce acceleration noise
            double acceleration = Math.max(accelerationMagnitude - accelerationNoise, 0) * accelerationDirection;
            
            // Update speed and distance
            speed = clamp(speed + acceleration * elapsed, -0.25, 0.25);
            distance = clamp(distance + speed * elapsed, 0.25, 0.75);
            
            // Update Pokemon size based on distance
            pokemonImageScale = 1.0f - (float) distance;
            configurePokemon();
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
