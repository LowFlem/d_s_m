package dsm.vaulthunter.controller;

import android.Manifest;
import android.content.Context;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.graphics.drawable.Drawable;
import android.location.Location;
import android.location.LocationListener;
import android.location.LocationManager;
import android.os.Bundle;
import android.util.Log;
import android.view.View;
import android.widget.ImageButton;
import android.widget.TextView;
import android.widget.Toast;

import androidx.appcompat.app.AppCompatActivity;
import androidx.core.app.ActivityCompat;
import androidx.core.content.ContextCompat;

import org.osmdroid.api.IMapController;
import org.osmdroid.tileprovider.tilesource.TileSourceFactory;
import org.osmdroid.util.GeoPoint;
import org.osmdroid.views.MapView;
import org.osmdroid.views.overlay.Marker;
import org.osmdroid.views.overlay.mylocation.GpsMyLocationProvider;
import org.osmdroid.views.overlay.mylocation.MyLocationNewOverlay;

import java.util.ArrayList;
import java.util.List;

import dsm.vaulthunter.model.AppController;
import dsm.vaulthunter.model.Treasure;
import dsm.vaulthunter.model.TreasureAppearance;
import dsm.vaulthunter.model.Vault;
import dsm.vaulthunter.util.MapConfigUtils;
import teste.lucasvegi.pokemongooffline.R;

/**
 * Main map activity for the Vault Hunter game
 */
public class MapActivity extends AppCompatActivity implements LocationListener {

    private static final String TAG = "VH_MapActivity";
    private static final int LOCATION_PERMISSION_REQUEST_CODE = 1;
    private static final float MIN_DISTANCE_TO_TREASURE = 50.0f; // meters
    private static final float TREASURE_SPAWN_RADIUS = 200.0f; // meters
    
    private MapView map;
    private IMapController mapController;
    private MyLocationNewOverlay locationOverlay;
    private LocationManager locationManager;
    private GeoPoint currentLocation;
    
    // Track Treasures and Vaults on map
    private List<Marker> treasureMarkers = new ArrayList<>();
    private List<Marker> vaultMarkers = new ArrayList<>();
    
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        
        // Initialize osmdroid configuration
        Context ctx = getApplicationContext();
        MapConfigUtils.initializeOsmDroid(ctx);
        
        setContentView(R.layout.activity_map);
        
        // Set up the map
        map = findViewById(R.id.mapa);
        map.setTileSource(TileSourceFactory.MAPNIK);
        map.setMultiTouchControls(true);
        
        // Set initial zoom level
        mapController = map.getController();
        mapController.setZoom(18.0);
        
        // Set up location tracking
        setupLocationTracking();
        
        // Update UI with user info
        updateUserInfo();
        
        // Request permissions if needed
        requestLocationPermission();
    }
    
    @Override
    protected void onResume() {
        super.onResume();
        map.onResume();
        
        // Start location updates
        if (ActivityCompat.checkSelfPermission(this, Manifest.permission.ACCESS_FINE_LOCATION) == 
                PackageManager.PERMISSION_GRANTED) {
            locationManager.requestLocationUpdates(
                    LocationManager.GPS_PROVIDER, 
                    5000,   // 5 seconds
                    10,     // 10 meters
                    this);
        }
    }
    
    @Override
    protected void onPause() {
        super.onPause();
        map.onPause();
        
        // Stop location updates
        locationManager.removeUpdates(this);
    }
    
    private void setupLocationTracking() {
        // Set up the location overlay
        GpsMyLocationProvider provider = new GpsMyLocationProvider(this);
        locationOverlay = new MyLocationNewOverlay(provider, map);
        locationOverlay.enableMyLocation();
        locationOverlay.enableFollowLocation();
        map.getOverlays().add(locationOverlay);
        
        // Initialize location manager
        locationManager = (LocationManager) getSystemService(Context.LOCATION_SERVICE);
    }
    
    private void updateUserInfo() {
        // Update the user name in the UI
        TextView txtNomeUser = findViewById(R.id.txtNomeUser);
        String userName = AppController.getInstance().getLoggedInUser().getName();
        txtNomeUser.setText(userName);
        
        // Update user avatar based on gender
        ImageButton btnProfile = findViewById(R.id.botaoPerfil);
        char gender = AppController.getInstance().getLoggedInUser().getGender();
        if (gender == 'F') {
            btnProfile.setImageResource(R.drawable.female_profile);
        } else {
            btnProfile.setImageResource(R.drawable.male_profile);
        }
    }
    
    private void requestLocationPermission() {
        if (ContextCompat.checkSelfPermission(this, Manifest.permission.ACCESS_FINE_LOCATION)
                != PackageManager.PERMISSION_GRANTED) {
            ActivityCompat.requestPermissions(this,
                    new String[]{Manifest.permission.ACCESS_FINE_LOCATION},
                    LOCATION_PERMISSION_REQUEST_CODE);
        } else {
            // Permission already granted
            onLocationPermissionGranted();
        }
    }
    
    private void onLocationPermissionGranted() {
        // Start getting location updates
        try {
            if (locationManager.isProviderEnabled(LocationManager.GPS_PROVIDER)) {
                locationManager.requestLocationUpdates(
                        LocationManager.GPS_PROVIDER,
                        5000,   // 5 seconds
                        10,     // 10 meters
                        this);
                
                // Try to get last known location
                Location lastLocation = locationManager.getLastKnownLocation(LocationManager.GPS_PROVIDER);
                if (lastLocation != null) {
                    onLocationChanged(lastLocation);
                }
            } else {
                Toast.makeText(this, "Please enable GPS", Toast.LENGTH_LONG).show();
            }
        } catch (SecurityException e) {
            Log.e(TAG, "Security exception: " + e.getMessage());
        }
    }
    
    @Override
    public void onRequestPermissionsResult(int requestCode, String[] permissions, int[] grantResults) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults);
        if (requestCode == LOCATION_PERMISSION_REQUEST_CODE) {
            if (grantResults.length > 0 && grantResults[0] == PackageManager.PERMISSION_GRANTED) {
                onLocationPermissionGranted();
            } else {
                Toast.makeText(this, R.string.permissions_location_required, 
                        Toast.LENGTH_LONG).show();
            }
        }
    }
    
    @Override
    public void onLocationChanged(Location location) {
        // Update current location
        currentLocation = new GeoPoint(location.getLatitude(), location.getLongitude());
        
        // Center map on current location
        mapController.setCenter(currentLocation);
        
        // Update user's last known location
        AppController.getInstance().getLoggedInUser().setLastLatitude(String.valueOf(location.getLatitude()));
        AppController.getInstance().getLoggedInUser().setLastLongitude(String.valueOf(location.getLongitude()));
        
        // Generate treasures around the player
        generateTreasures();
        
        // Generate vaults around the player
        generateVaults();
        
        // Check if player is close to any treasure or vault
        checkProximity();
    }
    
    private void generateTreasures() {
        // Only generate new treasures if we don't have any
        if (treasureMarkers.isEmpty()) {
            // Generate 3-6 random treasures around the player
            int count = (int)(Math.random() * 4) + 3;
            
            // Generate treasure appearances
            List<TreasureAppearance> appearances = AppController.getInstance().generateTreasureAppearances(
                    currentLocation.getLatitude(),
                    currentLocation.getLongitude(),
                    count,
                    TREASURE_SPAWN_RADIUS
            );
            
            // Add markers for each appearance
            for (TreasureAppearance appearance : appearances) {
                addTreasureMarker(appearance);
            }
            
            // Refresh the map to show new markers
            map.invalidate();
        }
    }
    
    private void addTreasureMarker(TreasureAppearance appearance) {
        // Create marker at the appearance location
        GeoPoint treasureLocation = new GeoPoint(
                appearance.getLatitude(),
                appearance.getLongitude()
        );
        
        // Create a marker for the treasure
        Marker treasureMarker = new Marker(map);
        treasureMarker.setPosition(treasureLocation);
        treasureMarker.setAnchor(Marker.ANCHOR_CENTER, Marker.ANCHOR_BOTTOM);
        treasureMarker.setTitle(appearance.getTreasure().getName());
        
        // Set icon based on treasure type
        int iconResourceId;
        Treasure treasure = appearance.getTreasure();
        
        // Try to get icon for this treasure number
        iconResourceId = getResources().getIdentifier(
                "vh_treasure_" + treasure.getNumber(), "drawable", getPackageName());
        
        if (iconResourceId != 0) {
            Drawable icon = ContextCompat.getDrawable(this, iconResourceId);
            treasureMarker.setIcon(icon);
        } else {
            // Fallback icon based on treasure type
            int typeId = treasure.getPrimaryType().getId();
            iconResourceId = getResources().getIdentifier(
                    "vh_type_" + typeId, "drawable", getPackageName());
            
            if (iconResourceId != 0) {
                treasureMarker.setIcon(ContextCompat.getDrawable(this, iconResourceId));
            } else {
                // Final fallback
                treasureMarker.setIcon(ContextCompat.getDrawable(this, R.drawable.p1));
            }
        }
        
        // Add marker to the map
        map.getOverlays().add(treasureMarker);
        treasureMarkers.add(treasureMarker);
        
        // Store appearance data with marker
        treasureMarker.setRelatedObject(appearance);
        
        // Set up marker click listener
        treasureMarker.setOnMarkerClickListener((marker, mapView) -> {
            onTreasureMarkerClick(marker);
            return true;
        });
    }
    
    private void generateVaults() {
        // Generate vaults less frequently than treasures
        if (vaultMarkers.isEmpty() && Math.random() < 0.3) { // 30% chance
            // Generate 1-2 vaults
            int count = (int)(Math.random() * 2) + 1;
            
            // Use a larger radius for vaults
            float vaultRadius = TREASURE_SPAWN_RADIUS * 1.5f;
            
            // Generate vaults
            List<Vault> vaults = AppController.getInstance().generateVaults(
                    currentLocation.getLatitude(),
                    currentLocation.getLongitude(),
                    count,
                    vaultRadius,
                    "LOCAL" // Default region
            );
            
            // Add markers for each vault
            for (Vault vault : vaults) {
                addVaultMarker(vault);
            }
            
            // Refresh the map
            map.invalidate();
        }
    }
    
    private void addVaultMarker(Vault vault) {
        // Create marker at the vault location
        GeoPoint vaultLocation = new GeoPoint(
                vault.getLatitude(),
                vault.getLongitude()
        );
        
        // Create a marker for the vault
        Marker vaultMarker = new Marker(map);
        vaultMarker.setPosition(vaultLocation);
        vaultMarker.setAnchor(Marker.ANCHOR_CENTER, Marker.ANCHOR_BOTTOM);
        
        // Set title based on variant
        String title = vault.getVariant().name() + " Vault";
        vaultMarker.setTitle(title);
        
        // Set icon based on vault variant
        int iconResourceId = getResources().getIdentifier(
                vault.getVaultDrawableResource(), "drawable", getPackageName());
        
        if (iconResourceId != 0) {
            Drawable icon = ContextCompat.getDrawable(this, iconResourceId);
            vaultMarker.setIcon(icon);
        } else {
            // Fallback icon
            vaultMarker.setIcon(ContextCompat.getDrawable(this, R.drawable.dsm_bronze_chest));
        }
        
        // Add marker to the map
        map.getOverlays().add(vaultMarker);
        vaultMarkers.add(vaultMarker);
        
        // Store vault data with marker
        vaultMarker.setRelatedObject(vault);
        
        // Set up marker click listener
        vaultMarker.setOnMarkerClickListener((marker, mapView) -> {
            onVaultMarkerClick(marker);
            return true;
        });
    }
    
    private void checkProximity() {
        if (currentLocation != null) {
            // Check treasures
            for (Marker marker : treasureMarkers) {
                float[] results = new float[1];
                Location.distanceBetween(
                        currentLocation.getLatitude(),
                        currentLocation.getLongitude(),
                        marker.getPosition().getLatitude(),
                        marker.getPosition().getLongitude(),
                        results);
                
                // If player is within collection range
                if (results[0] <= MIN_DISTANCE_TO_TREASURE) {
                    // Add a visual indicator that treasure is in range
                    marker.setAlpha(1.0f);  // Full opacity
                } else {
                    // Slightly transparent when out of range
                    marker.setAlpha(0.7f);
                }
            }
            
            // Check vaults
            for (Marker marker : vaultMarkers) {
                float[] results = new float[1];
                Location.distanceBetween(
                        currentLocation.getLatitude(),
                        currentLocation.getLongitude(),
                        marker.getPosition().getLatitude(),
                        marker.getPosition().getLongitude(),
                        results);
                
                // If player is within collection range
                if (results[0] <= MIN_DISTANCE_TO_TREASURE) {
                    // Add a visual indicator that vault is in range
                    marker.setAlpha(1.0f);  // Full opacity
                } else {
                    // Slightly transparent when out of range
                    marker.setAlpha(0.7f);
                }
            }
            
            map.invalidate();
        }
    }
    
    private void onTreasureMarkerClick(Marker marker) {
        if (currentLocation != null) {
            // Calculate distance to treasure
            float[] results = new float[1];
            Location.distanceBetween(
                    currentLocation.getLatitude(),
                    currentLocation.getLongitude(),
                    marker.getPosition().getLatitude(),
                    marker.getPosition().getLongitude(),
                    results);
            
            TreasureAppearance appearance = (TreasureAppearance) marker.getRelatedObject();
            
            // Check if player is close enough
            if (results[0] <= MIN_DISTANCE_TO_TREASURE) {
                // Start treasure collection activity
                startTreasureCollectionActivity(appearance);
            } else {
                // Too far, show distance message
                int distanceRemaining = Math.round(results[0] - MIN_DISTANCE_TO_TREASURE);
                String message = "You're too far from this treasure! Get " + 
                        distanceRemaining + " meters closer to collect it.";
                
                Toast.makeText(this, message, Toast.LENGTH_LONG).show();
            }
        }
    }
    
    private void onVaultMarkerClick(Marker marker) {
        if (currentLocation != null) {
            // Calculate distance to vault
            float[] results = new float[1];
            Location.distanceBetween(
                    currentLocation.getLatitude(),
                    currentLocation.getLongitude(),
                    marker.getPosition().getLatitude(),
                    marker.getPosition().getLongitude(),
                    results);
            
            Vault vault = (Vault) marker.getRelatedObject();
            
            // Check if player is close enough
            if (results[0] <= MIN_DISTANCE_TO_TREASURE) {
                // In a complete implementation, this would open the vault activity
                // For now, just show a message
                String message = "You found a " + vault.getVariant().name() + 
                        " vault worth " + vault.getTokenAmount() + " tokens!";
                
                Toast.makeText(this, message, Toast.LENGTH_LONG).show();
                
                // Remove the vault marker from the map
                map.getOverlays().remove(marker);
                vaultMarkers.remove(marker);
                map.invalidate();
                
                // Give XP to the user
                boolean leveledUp = AppController.getInstance().addExperiencePoints("vault");
                if (leveledUp) {
                    Toast.makeText(this, "Level up! You are now level " + 
                            AppController.getInstance().getLoggedInUser().getLevel(), 
                            Toast.LENGTH_LONG).show();
                }
            } else {
                // Too far, show distance message
                int distanceRemaining = Math.round(results[0] - MIN_DISTANCE_TO_TREASURE);
                String message = "You're too far from this vault! Get " + 
                        distanceRemaining + " meters closer to open it.";
                
                Toast.makeText(this, message, Toast.LENGTH_LONG).show();
            }
        }
    }
    
    private void startTreasureCollectionActivity(TreasureAppearance appearance) {
        Intent intent = new Intent(this, TreasureCollectionActivity.class);
        intent.putExtra("treasure_appearance", appearance);
        // Add current location for recording collection location
        intent.putExtra("latitude", currentLocation.getLatitude());
        intent.putExtra("longitude", currentLocation.getLongitude());
        startActivity(intent);
    }
    
    // Navigation methods
    public void clickProfile(View v) {
        Intent intent = new Intent(this, ProfileActivity.class);
        startActivity(intent);
    }
    
    public void clickTreasureDex(View v) {
        Intent intent = new Intent(this, TreasureDexActivity.class);
        startActivity(intent);
    }
    
    public void clickTreasureMap(View v) {
        Intent intent = new Intent(this, TreasureMapActivity.class);
        startActivity(intent);
    }
    
    public void clickVault(View v) {
        Intent intent = new Intent(this, VaultActivity.class);
        startActivity(intent);
    }
    
    // Required LocationListener methods
    @Override
    public void onStatusChanged(String provider, int status, Bundle extras) {
        // Not used in API 29+
    }
    
    @Override
    public void onProviderEnabled(String provider) {
        if (provider.equals(LocationManager.GPS_PROVIDER)) {
            Toast.makeText(this, "GPS enabled", Toast.LENGTH_SHORT).show();
        }
    }
    
    @Override
    public void onProviderDisabled(String provider) {
        if (provider.equals(LocationManager.GPS_PROVIDER)) {
            Toast.makeText(this, "GPS disabled - please enable it to play", Toast.LENGTH_LONG).show();
        }
    }
}