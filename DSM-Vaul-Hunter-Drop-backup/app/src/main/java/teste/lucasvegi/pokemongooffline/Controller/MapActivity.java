package teste.lucasvegi.pokemongooffline.Controller;

import android.Manifest;
import android.content.Context;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.graphics.drawable.Drawable;
import android.location.Location;
import android.location.LocationListener;
import android.location.LocationManager;
import android.os.Bundle;
import android.preference.PreferenceManager;
import android.util.Log;
import android.view.View;
import android.widget.ImageButton;
import android.widget.TextView;
import android.widget.Toast;

import androidx.appcompat.app.AppCompatActivity;
import androidx.core.app.ActivityCompat;
import androidx.core.content.ContextCompat;

import org.osmdroid.api.IMapController;
import org.osmdroid.config.Configuration;
import org.osmdroid.tileprovider.tilesource.TileSourceFactory;
import org.osmdroid.util.GeoPoint;
import org.osmdroid.views.MapView;
import org.osmdroid.views.overlay.Marker;
import org.osmdroid.views.overlay.mylocation.GpsMyLocationProvider;
import org.osmdroid.views.overlay.mylocation.MyLocationNewOverlay;

import java.io.File;
import java.util.ArrayList;
import java.util.List;
import java.util.Random;

import teste.lucasvegi.pokemongooffline.Model.ControladoraFachadaSingleton;
import teste.lucasvegi.pokemongooffline.Model.Pokemon;
import teste.lucasvegi.pokemongooffline.R;
import teste.lucasvegi.pokemongooffline.Util.MapConfigUtils;

public class MapActivity extends AppCompatActivity implements LocationListener {

    private static final String TAG = "MapActivity";
    private static final int LOCATION_PERMISSION_REQUEST_CODE = 1;
    private static final float MIN_DISTANCE_TO_POKEMON = 50.0f; // meters
    private static final float POKEMON_SPAWN_RADIUS = 200.0f; // meters
    
    private MapView map;
    private IMapController mapController;
    private MyLocationNewOverlay locationOverlay;
    private LocationManager locationManager;
    private GeoPoint currentLocation;
    
    // Track Pokemon on map
    private List<Marker> pokemonMarkers = new ArrayList<>();
    private Random random = new Random();
    
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        
        // Initialize osmdroid configuration properly
        Context ctx = getApplicationContext();
        MapConfigUtils.initializeOsmDroid(ctx);
        
        // Handle OSM cache permissions
        handleOsmCachePermissions();
        
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
    
    /**
     * Handles OpenStreetMap tile caching and permission issues
     */
    private void handleOsmCachePermissions() {
        // For Android 6+ (API 23+), we need to request runtime permissions
        if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.M) {
            if (ActivityCompat.checkSelfPermission(this, Manifest.permission.WRITE_EXTERNAL_STORAGE) != 
                    PackageManager.PERMISSION_GRANTED) {
                // We don't have permission, but we're already handling location permissions explicitly
                // So we'll just log this but continue (osmdroid handles internal cache if external is unavailable)
                Log.w(TAG, "No WRITE_EXTERNAL_STORAGE permission, osmdroid will use internal cache");
            }
        }
        
        // Set up a specific cache path for tiles
        File osmCacheDir = new File(getCacheDir(), "osmdroid");
        File tileCache = new File(osmCacheDir, "tiles");
        if (!tileCache.exists()) {
            boolean created = tileCache.mkdirs();
            if (!created) {
                Log.w(TAG, "Failed to create tile cache directory");
            }
        }
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
        String userName = ControladoraFachadaSingleton.getInstance().getUsuarioLogado().getNome();
        txtNomeUser.setText(userName);
        
        // Update user avatar based on gender
        ImageButton btnProfile = findViewById(R.id.botaoPerfil);
        char gender = ControladoraFachadaSingleton.getInstance().getUsuarioLogado().getSexo();
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
                Toast.makeText(this, getString(R.string.permissions_location_required), 
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
        
        // Generate pokemon around the player
        generatePokemon();
        
        // Check if player is close to any pokemon
        checkPokemonProximity();
    }
    
    private void generatePokemon() {
        // Only generate new pokemon if we don't have any
        if (pokemonMarkers.isEmpty()) {
            // Generate 3-6 random pokemon around the player
            int numPokemon = random.nextInt(4) + 3;
            
            for (int i = 0; i < numPokemon; i++) {
                // Generate random point around the player
                double radiusInDegrees = POKEMON_SPAWN_RADIUS / 111320f; // 1 degree is approximately 111320 meters
                
                double u = random.nextDouble();
                double v = random.nextDouble();
                double w = radiusInDegrees * Math.sqrt(u);
                double t = 2 * Math.PI * v;
                double x = w * Math.cos(t);
                double y = w * Math.sin(t);
                
                // Adjust for latitude's distortion of longitude distance
                double new_x = x / Math.cos(Math.toRadians(currentLocation.getLatitude()));
                
                double newLat = currentLocation.getLatitude() + y;
                double newLon = currentLocation.getLongitude() + new_x;
                
                // Create a new GeoPoint for the pokemon
                GeoPoint pokemonLocation = new GeoPoint(newLat, newLon);
                
                // Get a random pokemon from the controller
                Pokemon pokemon = ControladoraFachadaSingleton.getInstance().sorteiaPokemon();
                if (pokemon != null) {
                    // Create a marker for the pokemon
                    Marker pokemonMarker = new Marker(map);
                    pokemonMarker.setPosition(pokemonLocation);
                    pokemonMarker.setAnchor(Marker.ANCHOR_CENTER, Marker.ANCHOR_BOTTOM);
                    pokemonMarker.setTitle(pokemon.getNome());
                    
                    // Set icon based on treasure type or chest variant
                    int iconResourceId;
                    
                    // Check if it's a chest (DSMLimboVault)
                    if (pokemon.getNumero() >= 900) {  // Using numbers >= 900 for chests
                        // Use the appropriate chest image based on the number
                        switch (pokemon.getNumero()) {
                            case 901:  // Bronze
                                iconResourceId = getResources().getIdentifier(
                                    "dsm_bronze_chest", "drawable", getPackageName());
                                break;
                            case 902:  // Silver
                                iconResourceId = getResources().getIdentifier(
                                    "dsm_silver_chest", "drawable", getPackageName());
                                break;
                            case 903:  // Gold
                                iconResourceId = getResources().getIdentifier(
                                    "dsm_gold_chest", "drawable", getPackageName());
                                break;
                            default:
                                iconResourceId = getResources().getIdentifier(
                                    "dsm_bronze_chest", "drawable", getPackageName());
                        }
                    } else {
                        // Regular Pokemon image
                        iconResourceId = getResources().getIdentifier(
                            "p" + pokemon.getNumero(), "drawable", getPackageName());
                    }
                    if (iconResourceId != 0) {
                        Drawable icon = ContextCompat.getDrawable(this, iconResourceId);
                        pokemonMarker.setIcon(icon);
                    } else {
                        // Fallback icon - depending on whether it's a chest or pokemon
                        if (pokemon.getNumero() >= 900) {
                            pokemonMarker.setIcon(ContextCompat.getDrawable(this, R.drawable.dsm_bronze_chest));
                        } else {
                            pokemonMarker.setIcon(ContextCompat.getDrawable(this, R.drawable.p1));
                        }
                    }
                    
                    // Add marker to the map
                    map.getOverlays().add(pokemonMarker);
                    pokemonMarkers.add(pokemonMarker);
                    
                    // Set tag with pokemon data
                    pokemonMarker.setRelatedObject(pokemon);
                    
                    // Set up marker click listener
                    pokemonMarker.setOnMarkerClickListener((marker, mapView) -> {
                        onPokemonMarkerClick(marker);
                        return true;
                    });
                }
            }
            
            // Refresh the map to show new markers
            map.invalidate();
        }
    }
    
    private void checkPokemonProximity() {
        if (currentLocation != null) {
            for (Marker marker : pokemonMarkers) {
                float[] results = new float[1];
                Location.distanceBetween(
                        currentLocation.getLatitude(),
                        currentLocation.getLongitude(),
                        marker.getPosition().getLatitude(),
                        marker.getPosition().getLongitude(),
                        results);
                
                // If player is within capture range
                if (results[0] <= MIN_DISTANCE_TO_POKEMON) {
                    // Add a visual indicator that pokemon is in range
                    marker.setAlpha(1.0f);  // Full opacity
                } else {
                    // Slightly transparent when out of range
                    marker.setAlpha(0.7f);
                }
            }
            map.invalidate();
        }
    }
    
    private void onPokemonMarkerClick(Marker marker) {
        if (currentLocation != null) {
            // Calculate distance to pokemon/chest
            float[] results = new float[1];
            Location.distanceBetween(
                    currentLocation.getLatitude(),
                    currentLocation.getLongitude(),
                    marker.getPosition().getLatitude(),
                    marker.getPosition().getLongitude(),
                    results);
            
            Pokemon pokemon = (Pokemon) marker.getRelatedObject();
            
            // Check if it's a treasure chest (special Pokemon with number >= 900)
            boolean isChest = pokemon.getNumero() >= 900;
            
            // Check if player is close enough
            if (results[0] <= MIN_DISTANCE_TO_POKEMON) {
                if (isChest) {
                    // For chests, we need special handling
                    handleChestInteraction(pokemon, marker.getPosition());
                } else {
                    // Regular Pokemon capture
                    startCaptureActivity(pokemon);
                }
            } else {
                // Too far, show distance message
                int distanceRemaining = Math.round(results[0] - MIN_DISTANCE_TO_POKEMON);
                String message;
                
                if (isChest) {
                    message = "You're too far from this treasure chest! Get " + 
                            distanceRemaining + " meters closer to open it.";
                } else {
                    message = getString(R.string.pokemon_too_far, pokemon.getNome()) +
                            " " + getString(R.string.distance_remaining, distanceRemaining);
                }
                
                Toast.makeText(this, message, Toast.LENGTH_LONG).show();
            }
        }
    }
    
    private void handleChestInteraction(Pokemon chestPokemon, GeoPoint position) {
        // Create a real DSMLimboVault object from the Pokemon representation
        DSMLimboVault.ChestVariant variant;
        switch (chestPokemon.getNumero()) {
            case 901:
                variant = DSMLimboVault.ChestVariant.BRONZE;
                break;
            case 902:
                variant = DSMLimboVault.ChestVariant.SILVER;
                break;
            case 903:
                variant = DSMLimboVault.ChestVariant.GOLD;
                break;
            default:
                variant = DSMLimboVault.ChestVariant.BRONZE;
        }
        
        // Create a chest object with a unique ID
        DSMLimboVault chest = new DSMLimboVault(
                "CHEST-" + System.currentTimeMillis(),
                "LOCAL",  // Region ID
                variant,
                position.getLatitude(),
                position.getLongitude());
        
        // For now, just show a success message - in the future this would open TreasureHuntActivity
        String message = "You found a " + variant.name() + " chest worth " + 
                chest.getTokenAmount() + " tokens!";
        Toast.makeText(this, message, Toast.LENGTH_LONG).show();
        
        // Remove the chest marker from the map
        for (Marker marker : pokemonMarkers) {
            Pokemon p = (Pokemon) marker.getRelatedObject();
            if (p == chestPokemon) {
                map.getOverlays().remove(marker);
                pokemonMarkers.remove(marker);
                break;
            }
        }
        map.invalidate();
    }
    
    private void startCaptureActivity(Pokemon pokemon) {
        Intent intent = new Intent(this, CapturaActivity.class);
        intent.putExtra("pkmnSelvagem", pokemon);
        // Add current location for recording capture location
        intent.putExtra("latitude", currentLocation.getLatitude());
        intent.putExtra("longitude", currentLocation.getLongitude());
        startActivity(intent);
    }
    
    // Navigation methods
    public void clickPerfil(View v) {
        Intent it = new Intent(this, PerfilActivity.class);
        startActivity(it);
    }
    
    public void clickPokedex(View v) {
        Intent it = new Intent(this, PokedexActivity.class);
        startActivity(it);
    }
    
    public void clickMapaCaptura(View v) {
        Intent it = new Intent(this, MapCapturasActivity.class);
        startActivity(it);
    }
    
    public void clickOvo(View v) {
        Intent it = new Intent(this, OvosActivity.class);
        startActivity(it);
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