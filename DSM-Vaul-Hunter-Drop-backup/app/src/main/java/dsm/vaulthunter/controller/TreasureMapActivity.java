package dsm.vaulthunter.controller;

import android.os.Bundle;
import android.view.View;
import android.widget.TextView;
import android.widget.Toast;

import androidx.appcompat.app.AppCompatActivity;

import org.osmdroid.api.IMapController;
import org.osmdroid.tileprovider.tilesource.TileSourceFactory;
import org.osmdroid.util.GeoPoint;
import org.osmdroid.views.MapView;
import org.osmdroid.views.overlay.Marker;
import org.osmdroid.views.overlay.Polygon;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

import dsm.vaulthunter.model.AppController;
import dsm.vaulthunter.model.CollectedTreasure;
import dsm.vaulthunter.model.Treasure;
import dsm.vaulthunter.model.User;
import dsm.vaulthunter.util.MapConfigUtils;
import teste.lucasvegi.pokemongooffline.R;

/**
 * Activity to display the map of collected treasures
 */
public class TreasureMapActivity extends AppCompatActivity {
    private static final String TAG = "TreasureMapActivity";
    
    private MapView map;
    private IMapController mapController;
    private User user;
    
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        
        // Initialize OSM configuration
        MapConfigUtils.initializeOsmDroid(getApplicationContext());
        
        setContentView(R.layout.activity_treasure_map);
        
        // Get current user
        user = AppController.getInstance().getLoggedInUser();
        
        // Initialize the map
        initializeMap();
        
        // Load user's collected treasures
        loadCollectedTreasures();
        
        // Update the stats
        updateStats();
    }
    
    private void initializeMap() {
        // Set up the map
        map = findViewById(R.id.treasureMap);
        map.setTileSource(TileSourceFactory.MAPNIK);
        map.setMultiTouchControls(true);
        map.setBuiltInZoomControls(true);
        map.setMaxZoomLevel(19.0);
        
        // Set initial zoom level
        mapController = map.getController();
        mapController.setZoom(6.0);
        
        // Set initial position to a default location (or user's last known location)
        GeoPoint startPoint;
        if (user.getLastLatitude() != null && user.getLastLongitude() != null) {
            try {
                double lat = Double.parseDouble(user.getLastLatitude());
                double lon = Double.parseDouble(user.getLastLongitude());
                startPoint = new GeoPoint(lat, lon);
            } catch (NumberFormatException e) {
                // Use default location if parsing fails
                startPoint = new GeoPoint(40.7128, -74.0060); // NYC coordinates
            }
        } else {
            // Use default location
            startPoint = new GeoPoint(40.7128, -74.0060); // NYC coordinates
        }
        
        mapController.setCenter(startPoint);
    }
    
    private void loadCollectedTreasures() {
        // Get all collected treasures
        List<CollectedTreasure> collectedTreasures = user.getCollectedTreasures();
        
        if (collectedTreasures.isEmpty()) {
            Toast.makeText(this, "You haven't collected any treasures yet", Toast.LENGTH_SHORT).show();
            return;
        }
        
        // Add markers for each collection location
        for (CollectedTreasure collected : collectedTreasures) {
            addMarkerForTreasure(collected);
        }
        
        // Add heat map for collection density
        addCollectionHeatmap(collectedTreasures);
        
        // Refresh the map
        map.invalidate();
    }
    
    private void addMarkerForTreasure(CollectedTreasure treasure) {
        double latitude = treasure.getLatitude();
        double longitude = treasure.getLongitude();
        
        // Skip if coordinates are invalid
        if (latitude == 0 && longitude == 0) {
            return;
        }
        
        // Create marker at collection location
        Marker marker = new Marker(map);
        marker.setPosition(new GeoPoint(latitude, longitude));
        marker.setAnchor(Marker.ANCHOR_CENTER, Marker.ANCHOR_BOTTOM);
        
        // Set title and snippet
        marker.setTitle(treasure.getTreasure().getName());
        marker.setSnippet("Collected: " + new java.text.SimpleDateFormat("yyyy-MM-dd")
                .format(new java.util.Date(treasure.getCollectionTime())));
        
        // Set icon based on treasure type
        int iconResourceId = getResources().getIdentifier(
                "p" + treasure.getTreasure().getNumber(), "drawable", getPackageName());
        
        if (iconResourceId != 0) {
            marker.setIcon(getResources().getDrawable(iconResourceId));
        } else {
            // Fallback to a default icon
            marker.setIcon(getResources().getDrawable(R.drawable.p0));
        }
        
        // Store treasure data
        marker.setRelatedObject(treasure);
        
        // Add marker to the map
        map.getOverlays().add(marker);
    }
    
    private void addCollectionHeatmap(List<CollectedTreasure> treasures) {
        // Create a map to count treasures per area
        Map<String, Integer> areaCounts = new HashMap<>();
        
        // Process each treasure location
        for (CollectedTreasure treasure : treasures) {
            // Skip invalid coordinates
            if (treasure.getLatitude() == 0 && treasure.getLongitude() == 0) {
                continue;
            }
            
            // Create a grid cell key (rounded to 0.01 degrees, roughly 1km)
            String key = String.format("%.2f,%.2f", 
                    Math.round(treasure.getLatitude() * 100) / 100.0, 
                    Math.round(treasure.getLongitude() * 100) / 100.0);
            
            // Increment the count for this area
            areaCounts.put(key, areaCounts.getOrDefault(key, 0) + 1);
        }
        
        // Create polygons for areas with multiple treasures
        for (Map.Entry<String, Integer> entry : areaCounts.entrySet()) {
            if (entry.getValue() > 1) {
                String[] coords = entry.getKey().split(",");
                double lat = Double.parseDouble(coords[0]);
                double lon = Double.parseDouble(coords[1]);
                
                // Create a square polygon around the coordinates
                Polygon polygon = new Polygon();
                List<GeoPoint> points = new ArrayList<>();
                
                // Create a 0.01 degree square (approximately 1km)
                double halfSide = 0.005; // half of 0.01 degrees
                points.add(new GeoPoint(lat - halfSide, lon - halfSide));
                points.add(new GeoPoint(lat - halfSide, lon + halfSide));
                points.add(new GeoPoint(lat + halfSide, lon + halfSide));
                points.add(new GeoPoint(lat + halfSide, lon - halfSide));
                
                polygon.setPoints(points);
                
                // Set color based on count (more treasures = more intense color)
                int alpha = Math.min(255, 50 + (entry.getValue() * 25)); // Limit to 255
                polygon.setFillColor(android.graphics.Color.argb(alpha, 255, 0, 0));
                polygon.setStrokeColor(android.graphics.Color.RED);
                polygon.setStrokeWidth(1);
                
                // Add to map
                map.getOverlays().add(polygon);
            }
        }
    }
    
    private void updateStats() {
        // Update statistics labels
        TextView totalTreasuresText = findViewById(R.id.totalTreasures);
        TextView uniqueTreasuresText = findViewById(R.id.uniqueTreasures);
        
        // Count treasures
        int totalCount = user.getCollectedTreasures().size();
        
        // Count unique treasures
        int uniqueCount = 0;
        for (Treasure treasure : AppController.getInstance().getAllTreasures()) {
            if (user.getTreasureCount(treasure) > 0) {
                uniqueCount++;
            }
        }
        
        // Update UI
        totalTreasuresText.setText(String.valueOf(totalCount));
        uniqueTreasuresText.setText(String.valueOf(uniqueCount));
    }
    
    @Override
    protected void onResume() {
        super.onResume();
        map.onResume();
    }
    
    @Override
    protected void onPause() {
        super.onPause();
        map.onPause();
    }
    
    // Click handler for back button
    public void goBack(View view) {
        finish();
    }
}