package dsm.vaulthunter.controller;

import android.content.Intent;
import android.location.Location;
import android.net.Uri;
import android.os.Bundle;
import android.os.Handler;
import android.view.View;
import android.widget.Button;
import android.widget.ImageView;
import android.widget.LinearLayout;
import android.widget.TextView;
import android.widget.Toast;

import androidx.appcompat.app.AlertDialog;
import androidx.appcompat.app.AppCompatActivity;

import org.osmdroid.api.IMapController;
import org.osmdroid.tileprovider.tilesource.TileSourceFactory;
import org.osmdroid.util.GeoPoint;
import org.osmdroid.views.MapView;
import org.osmdroid.views.overlay.Marker;

import java.text.SimpleDateFormat;
import java.util.Date;
import java.util.Locale;

import dsm.vaulthunter.model.AppController;
import dsm.vaulthunter.model.DSMClient;
import dsm.vaulthunter.model.User;
import dsm.vaulthunter.model.Vault;
import dsm.vaulthunter.util.MapConfigUtils;
import teste.lucasvegi.pokemongooffline.R;

/**
 * Activity for displaying and interacting with a vault
 */
public class VaultActivity extends AppCompatActivity {
    private static final String TAG = "VaultActivity";
    
    private Vault vault;
    private User user;
    private Handler locationUpdateHandler;
    private Runnable locationUpdateRunnable;
    private boolean locationUpdatesActive = false;
    
    // UI elements
    private ImageView vaultImage;
    private TextView vaultName;
    private TextView vaultStatus;
    private TextView difficultyLevel;
    private TextView unlockTime;
    private TextView expiryTime;
    private TextView tokenReward;
    private TextView claimedByText;
    private TextView claimedTime;
    private LinearLayout claimedByLayout;
    private LinearLayout claimedTimeLayout;
    private TextView distanceText;
    private Button getDirectionsButton;
    private TextView actionDescription;
    private Button unlockButton;
    private Button claimButton;
    private Button withdrawButton;
    private MapView miniMap;
    
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_vault);
        
        // Get current user
        user = AppController.getInstance().getLoggedInUser();
        
        // Get vault from intent
        long vaultId = getIntent().getLongExtra("vaultId", -1);
        
        if (vaultId == -1) {
            Toast.makeText(this, "Error loading vault details", Toast.LENGTH_SHORT).show();
            finish();
            return;
        }
        
        // Load vault from database
        vault = AppController.getInstance().getVaultById(vaultId);
        
        if (vault == null) {
            Toast.makeText(this, "Vault not found", Toast.LENGTH_SHORT).show();
            finish();
            return;
        }
        
        // Initialize views
        initializeViews();
        
        // Initialize the map
        initializeMap();
        
        // Update UI with vault data
        updateUI();
        
        // Start location updates
        startLocationUpdates();
    }
    
    @Override
    protected void onResume() {
        super.onResume();
        
        // Resume location updates
        startLocationUpdates();
        
        // Refresh vault data in case it was updated elsewhere
        if (vault != null) {
            vault = AppController.getInstance().getVaultById(vault.getId());
            updateUI();
        }
        
        // Resume map
        if (miniMap != null) {
            miniMap.onResume();
        }
    }
    
    @Override
    protected void onPause() {
        super.onPause();
        
        // Pause location updates
        stopLocationUpdates();
        
        // Pause map
        if (miniMap != null) {
            miniMap.onPause();
        }
    }
    
    private void initializeViews() {
        vaultImage = findViewById(R.id.vaultImage);
        vaultName = findViewById(R.id.vaultName);
        vaultStatus = findViewById(R.id.vaultStatus);
        difficultyLevel = findViewById(R.id.difficultyLevel);
        unlockTime = findViewById(R.id.unlockTime);
        expiryTime = findViewById(R.id.expiryTime);
        tokenReward = findViewById(R.id.tokenReward);
        claimedByText = findViewById(R.id.claimedByText);
        claimedTime = findViewById(R.id.claimedTime);
        claimedByLayout = findViewById(R.id.claimedByLayout);
        claimedTimeLayout = findViewById(R.id.claimedTimeLayout);
        distanceText = findViewById(R.id.distanceText);
        getDirectionsButton = findViewById(R.id.getDirectionsButton);
        actionDescription = findViewById(R.id.actionDescription);
        unlockButton = findViewById(R.id.unlockButton);
        claimButton = findViewById(R.id.claimButton);
        withdrawButton = findViewById(R.id.withdrawButton);
    }
    
    private void initializeMap() {
        // Initialize OSM configuration
        MapConfigUtils.initializeOsmDroid(getApplicationContext());
        
        // Create a new MapView
        miniMap = new MapView(this);
        miniMap.setTileSource(TileSourceFactory.MAPNIK);
        miniMap.setMultiTouchControls(true);
        miniMap.setBuiltInZoomControls(false);
        
        // Add the map to the container
        findViewById(R.id.miniMapContainer).addView(miniMap);
        
        // Set initial position to vault location
        IMapController mapController = miniMap.getController();
        mapController.setZoom(15.0);
        
        GeoPoint vaultLocation = new GeoPoint(vault.getLatitude(), vault.getLongitude());
        mapController.setCenter(vaultLocation);
        
        // Add marker for vault
        Marker vaultMarker = new Marker(miniMap);
        vaultMarker.setPosition(vaultLocation);
        vaultMarker.setAnchor(Marker.ANCHOR_CENTER, Marker.ANCHOR_BOTTOM);
        vaultMarker.setTitle(vault.getName());
        
        // Set icon based on vault status
        int markerIcon;
        switch (vault.getStatus()) {
            case CLAIMED:
            case WITHDRAWN:
                markerIcon = R.drawable.dsm_gold_chest; // Use an opened chest icon
                break;
            case UNLOCKED:
                markerIcon = R.drawable.dsm_silver_chest; // Use an unlocked chest icon
                break;
            default:
                markerIcon = R.drawable.dsm_bronze_chest; // Use a locked chest icon
        }
        
        if (getResources().getDrawable(markerIcon) != null) {
            vaultMarker.setIcon(getResources().getDrawable(markerIcon));
        }
        
        // Add to map
        miniMap.getOverlays().add(vaultMarker);
        
        // Add user location marker if available
        if (user.getLastLatitude() != null && user.getLastLongitude() != null) {
            try {
                double userLat = Double.parseDouble(user.getLastLatitude());
                double userLon = Double.parseDouble(user.getLastLongitude());
                
                Marker userMarker = new Marker(miniMap);
                userMarker.setPosition(new GeoPoint(userLat, userLon));
                userMarker.setAnchor(Marker.ANCHOR_CENTER, Marker.ANCHOR_BOTTOM);
                userMarker.setTitle("Your Location");
                
                // Use a different icon for user
                userMarker.setIcon(getResources().getDrawable(android.R.drawable.ic_menu_mylocation));
                
                // Add to map
                miniMap.getOverlays().add(userMarker);
            } catch (NumberFormatException e) {
                // Ignore invalid coordinates
            }
        }
        
        // Refresh the map
        miniMap.invalidate();
    }
    
    private void updateUI() {
        // Set vault name and image
        vaultName.setText(vault.getName());
        
        // Set appropriate vault image based on status
        int vaultImageRes;
        switch (vault.getStatus()) {
            case CLAIMED:
            case WITHDRAWN:
                vaultImageRes = R.drawable.dsm_gold_chest;
                break;
            case UNLOCKED:
                vaultImageRes = R.drawable.dsm_silver_chest;
                break;
            case EXPIRED:
                vaultImageRes = R.drawable.dsm_bronze_chest; // Use a different image for expired
                break;
            default:
                vaultImageRes = R.drawable.dsm_bronze_chest;
        }
        vaultImage.setImageResource(vaultImageRes);
        
        // Set status text and color
        vaultStatus.setText(getStatusText(vault.getStatus()));
        
        int statusColor;
        switch (vault.getStatus()) {
            case UNLOCKED:
                statusColor = android.graphics.Color.parseColor("#FF9800"); // Orange
                break;
            case CLAIMED:
                statusColor = android.graphics.Color.parseColor("#4CAF50"); // Green
                break;
            case WITHDRAWN:
                statusColor = android.graphics.Color.parseColor("#9C27B0"); // Purple
                break;
            case EXPIRED:
                statusColor = android.graphics.Color.parseColor("#F44336"); // Red
                break;
            default:
                statusColor = android.graphics.Color.parseColor("#607D8B"); // Blue-grey
        }
        vaultStatus.setBackgroundColor(statusColor);
        
        // Set info card details
        String difficultyText;
        switch (vault.getDifficultyLevel()) {
            case 1:
                difficultyText = "Easy";
                break;
            case 2:
                difficultyText = "Medium";
                break;
            case 3:
                difficultyText = "Hard";
                break;
            case 4:
                difficultyText = "Expert";
                break;
            case 5:
                difficultyText = "Legendary";
                break;
            default:
                difficultyText = "Unknown";
        }
        difficultyLevel.setText(difficultyText);
        
        // Format dates
        SimpleDateFormat dateFormat = new SimpleDateFormat("MMM dd, yyyy hh:mm a", Locale.US);
        
        unlockTime.setText(dateFormat.format(new Date(vault.getUnlockTimestamp())));
        expiryTime.setText(dateFormat.format(new Date(vault.getExpiryTimestamp())));
        
        // Set token reward
        tokenReward.setText(vault.getTokenAmount() + " DSM");
        
        // Show/hide and set claimed info
        if (vault.getStatus() == Vault.VaultStatus.CLAIMED || 
            vault.getStatus() == Vault.VaultStatus.WITHDRAWN) {
            
            claimedByLayout.setVisibility(View.VISIBLE);
            claimedTimeLayout.setVisibility(View.VISIBLE);
            
            // Check if claimed by current user
            if (vault.getClaimedBy() != null && vault.getClaimedBy().getId() == user.getId()) {
                claimedByText.setText("You");
            } else if (vault.getClaimedBy() != null) {
                claimedByText.setText(vault.getClaimedBy().getName());
            } else {
                claimedByText.setText("Unknown");
            }
            
            claimedTime.setText(dateFormat.format(new Date(vault.getClaimedTimestamp())));
        } else {
            claimedByLayout.setVisibility(View.GONE);
            claimedTimeLayout.setVisibility(View.GONE);
        }
        
        // Update distance text if location available
        updateDistanceText();
        
        // Update action buttons based on vault status
        updateActionButtons();
    }
    
    private String getStatusText(Vault.VaultStatus status) {
        switch (status) {
            case LOCKED:
                return "LOCKED";
            case UNLOCKED:
                return "UNLOCKED";
            case CLAIMED:
                return "CLAIMED";
            case WITHDRAWN:
                return "WITHDRAWN";
            case EXPIRED:
                return "EXPIRED";
            default:
                return "UNKNOWN";
        }
    }
    
    private void updateDistanceText() {
        if (user.getLastLatitude() != null && user.getLastLongitude() != null) {
            try {
                double userLat = Double.parseDouble(user.getLastLatitude());
                double userLon = Double.parseDouble(user.getLastLongitude());
                
                double distance = vault.calculateDistance(userLat, userLon);
                
                // Format distance based on magnitude
                String distanceStr;
                if (distance < 1000) {
                    distanceStr = String.format(Locale.US, "%.0f meters", distance);
                } else {
                    distanceStr = String.format(Locale.US, "%.2f km", distance / 1000);
                }
                
                distanceText.setText("You are " + distanceStr + " away from this vault");
            } catch (NumberFormatException e) {
                distanceText.setText("Distance unknown");
            }
        } else {
            distanceText.setText("Location not available");
        }
    }
    
    private void updateActionButtons() {
        Vault.VaultStatus status = vault.getStatus();
        
        // Check if the vault is expired
        if (vault.checkExpired()) {
            status = Vault.VaultStatus.EXPIRED;
            
            // Update in database if status changed
            if (status != vault.getStatus()) {
                vault.setStatus(status);
                AppController.getInstance().updateVault(vault);
            }
        }
        
        // Update button visibility and text based on status
        switch (status) {
            case LOCKED:
                actionDescription.setText("This vault is currently locked. It will be available for unlocking on " + 
                        new SimpleDateFormat("MMM dd, yyyy hh:mm a", Locale.US)
                                .format(new Date(vault.getUnlockTimestamp())));
                
                unlockButton.setEnabled(false);
                unlockButton.setText("Locked");
                unlockButton.setVisibility(View.VISIBLE);
                claimButton.setVisibility(View.GONE);
                withdrawButton.setVisibility(View.GONE);
                break;
                
            case UNLOCKED:
                actionDescription.setText("You need to be within " + 
                        (int) vault.getRequiredDistance() + 
                        " meters of the vault to claim its contents.");
                
                boolean isInRange = false;
                if (user.getLastLatitude() != null && user.getLastLongitude() != null) {
                    try {
                        double userLat = Double.parseDouble(user.getLastLatitude());
                        double userLon = Double.parseDouble(user.getLastLongitude());
                        isInRange = vault.isUserWithinRange(userLat, userLon);
                    } catch (NumberFormatException e) {
                        // Do nothing
                    }
                }
                
                unlockButton.setVisibility(View.GONE);
                claimButton.setVisibility(View.VISIBLE);
                claimButton.setEnabled(isInRange);
                
                if (!isInRange) {
                    claimButton.setText("Get Closer to Claim");
                } else {
                    claimButton.setText("Claim Vault");
                }
                
                withdrawButton.setVisibility(View.GONE);
                break;
                
            case CLAIMED:
                boolean isClaimedByUser = vault.getClaimedBy() != null && 
                                         vault.getClaimedBy().getId() == user.getId();
                
                if (isClaimedByUser) {
                    actionDescription.setText("You have successfully claimed this vault! You can withdraw " + 
                            vault.getTokenAmount() + " DSM tokens to your wallet.");
                    
                    unlockButton.setVisibility(View.GONE);
                    claimButton.setVisibility(View.GONE);
                    withdrawButton.setVisibility(View.VISIBLE);
                    withdrawButton.setEnabled(true);
                } else {
                    actionDescription.setText("This vault has been claimed by another hunter.");
                    
                    unlockButton.setVisibility(View.GONE);
                    claimButton.setVisibility(View.GONE);
                    withdrawButton.setVisibility(View.GONE);
                }
                break;
                
            case WITHDRAWN:
                if (vault.getClaimedBy() != null && vault.getClaimedBy().getId() == user.getId()) {
                    actionDescription.setText("You have withdrawn " + vault.getTokenAmount() + 
                            " DSM tokens from this vault to your wallet.");
                } else {
                    actionDescription.setText("The tokens from this vault have been withdrawn by " +
                            (vault.getClaimedBy() != null ? vault.getClaimedBy().getName() : "another hunter"));
                }
                
                unlockButton.setVisibility(View.GONE);
                claimButton.setVisibility(View.GONE);
                withdrawButton.setVisibility(View.GONE);
                break;
                
            case EXPIRED:
                actionDescription.setText("This vault has expired and is no longer available.");
                
                unlockButton.setVisibility(View.GONE);
                claimButton.setVisibility(View.GONE);
                withdrawButton.setVisibility(View.GONE);
                break;
        }
    }
    
    private void startLocationUpdates() {
        if (locationUpdatesActive) {
            return;
        }
        
        locationUpdateHandler = new Handler();
        locationUpdateRunnable = new Runnable() {
            @Override
            public void run() {
                // In a real app, this would use GPS or network location
                // For this skeleton, we'll simulate location changes
                simulateLocationUpdate();
                
                // Schedule next update
                locationUpdateHandler.postDelayed(this, 10000); // Every 10 seconds
            }
        };
        
        // Start updates
        locationUpdateHandler.post(locationUpdateRunnable);
        locationUpdatesActive = true;
    }
    
    private void stopLocationUpdates() {
        if (locationUpdateHandler != null && locationUpdateRunnable != null) {
            locationUpdateHandler.removeCallbacks(locationUpdateRunnable);
            locationUpdatesActive = false;
        }
    }
    
    private void simulateLocationUpdate() {
        // This method simulates location updates
        // In a real app, you would get updates from LocationManager
        
        // For this skeleton, we'll use a simple approach:
        // If not already set, set the location to a point near the vault
        if (user.getLastLatitude() == null || user.getLastLongitude() == null) {
            // Set location 100-200 meters from the vault
            double bearing = Math.random() * 360; // Random direction
            double distance = 100 + Math.random() * 100; // 100-200 meters
            
            // Convert distance to degrees (approximate)
            double earthRadius = 6371000; // meters
            double latOffset = Math.cos(Math.toRadians(bearing)) * distance / earthRadius;
            double lonOffset = Math.sin(Math.toRadians(bearing)) * distance / 
                               (earthRadius * Math.cos(Math.toRadians(vault.getLatitude())));
            
            double newLat = vault.getLatitude() + Math.toDegrees(latOffset);
            double newLon = vault.getLongitude() + Math.toDegrees(lonOffset);
            
            user.setLastLatitude(String.valueOf(newLat));
            user.setLastLongitude(String.valueOf(newLon));
            
            // Save to database
            AppController.getInstance().updateUser(user);
            
            // Update UI
            updateDistanceText();
            updateActionButtons();
            
            // Add user marker to map
            addUserMarkerToMap(newLat, newLon);
        }
        // Otherwise, we could simulate the user moving closer to the vault over time
        // But for simplicity, we'll keep the position static in this skeleton
    }
    
    private void addUserMarkerToMap(double lat, double lon) {
        if (miniMap != null) {
            // Remove existing user markers
            for (int i = miniMap.getOverlays().size() - 1; i >= 0; i--) {
                if (miniMap.getOverlays().get(i) instanceof Marker) {
                    Marker m = (Marker) miniMap.getOverlays().get(i);
                    if ("Your Location".equals(m.getTitle())) {
                        miniMap.getOverlays().remove(i);
                    }
                }
            }
            
            // Add new user marker
            Marker userMarker = new Marker(miniMap);
            userMarker.setPosition(new GeoPoint(lat, lon));
            userMarker.setAnchor(Marker.ANCHOR_CENTER, Marker.ANCHOR_BOTTOM);
            userMarker.setTitle("Your Location");
            
            // Use a different icon for user
            userMarker.setIcon(getResources().getDrawable(android.R.drawable.ic_menu_mylocation));
            
            // Add to map
            miniMap.getOverlays().add(userMarker);
            
            // Refresh the map
            miniMap.invalidate();
        }
    }
    
    public void onUnlockClick(View view) {
        // Check if vault is available to unlock
        long currentTime = System.currentTimeMillis();
        
        if (vault.getStatus() == Vault.VaultStatus.LOCKED && 
            currentTime >= vault.getUnlockTimestamp()) {
            
            // Show confirmation dialog
            new AlertDialog.Builder(this)
                    .setTitle("Unlock Vault")
                    .setMessage("Are you sure you want to unlock this vault?")
                    .setPositiveButton("Unlock", (dialog, which) -> {
                        // Unlock the vault
                        if (vault.unlock()) {
                            // Save to database
                            AppController.getInstance().updateVault(vault);
                            
                            // Update UI
                            updateUI();
                            
                            Toast.makeText(VaultActivity.this, 
                                    "Vault unlocked successfully!", 
                                    Toast.LENGTH_SHORT).show();
                        } else {
                            Toast.makeText(VaultActivity.this, 
                                    "Failed to unlock vault", 
                                    Toast.LENGTH_SHORT).show();
                        }
                    })
                    .setNegativeButton("Cancel", null)
                    .show();
        } else {
            Toast.makeText(this, "Vault is not available for unlocking yet", 
                    Toast.LENGTH_SHORT).show();
        }
    }
    
    public void onClaimClick(View view) {
        // Check if vault is unlocked and user is within range
        if (vault.getStatus() != Vault.VaultStatus.UNLOCKED) {
            Toast.makeText(this, "Vault is not available for claiming", 
                    Toast.LENGTH_SHORT).show();
            return;
        }
        
        boolean isInRange = false;
        if (user.getLastLatitude() != null && user.getLastLongitude() != null) {
            try {
                double userLat = Double.parseDouble(user.getLastLatitude());
                double userLon = Double.parseDouble(user.getLastLongitude());
                isInRange = vault.isUserWithinRange(userLat, userLon);
            } catch (NumberFormatException e) {
                // Do nothing
            }
        }
        
        if (!isInRange) {
            Toast.makeText(this, "You need to be closer to the vault to claim it", 
                    Toast.LENGTH_SHORT).show();
            return;
        }
        
        // Show confirmation dialog
        new AlertDialog.Builder(this)
                .setTitle("Claim Vault")
                .setMessage("Are you sure you want to claim this vault and receive " + 
                        vault.getTokenAmount() + " DSM tokens?")
                .setPositiveButton("Claim", (dialog, which) -> {
                    // Claim the vault
                    if (vault.claim(user)) {
                        // Save to database
                        AppController.getInstance().updateVault(vault);
                        
                        // Add tokens to user's total
                        user.setTokensEarned(user.getTokensEarned() + vault.getTokenAmount());
                        AppController.getInstance().updateUser(user);
                        
                        // Update UI
                        updateUI();
                        
                        Toast.makeText(VaultActivity.this, 
                                "Vault claimed successfully!", 
                                Toast.LENGTH_SHORT).show();
                    } else {
                        Toast.makeText(VaultActivity.this, 
                                "Failed to claim vault", 
                                Toast.LENGTH_SHORT).show();
                    }
                })
                .setNegativeButton("Cancel", null)
                .show();
    }
    
    public void onWithdrawClick(View view) {
        // Check if vault is claimed and not withdrawn
        if (vault.getStatus() != Vault.VaultStatus.CLAIMED) {
            Toast.makeText(this, "Vault is not available for withdrawal", 
                    Toast.LENGTH_SHORT).show();
            return;
        }
        
        // Check if user is the one who claimed it
        if (vault.getClaimedBy() == null || vault.getClaimedBy().getId() != user.getId()) {
            Toast.makeText(this, "Only the hunter who claimed this vault can withdraw its tokens", 
                    Toast.LENGTH_SHORT).show();
            return;
        }
        
        // Check if connected to DSM network
        if (!DSMClient.getInstance().isConnected()) {
            Toast.makeText(this, "Not connected to DSM network", 
                    Toast.LENGTH_SHORT).show();
            return;
        }
        
        // Show confirmation dialog
        new AlertDialog.Builder(this)
                .setTitle("Withdraw Tokens")
                .setMessage("Are you sure you want to withdraw " + vault.getTokenAmount() + 
                        " DSM tokens to your wallet?")
                .setPositiveButton("Withdraw", (dialog, which) -> {
                    // Withdraw tokens
                    if (user.claimVaultTokens(vault)) {
                        // Save to database
                        AppController.getInstance().updateVault(vault);
                        AppController.getInstance().updateUser(user);
                        
                        // Update UI
                        updateUI();
                        
                        Toast.makeText(VaultActivity.this, 
                                vault.getTokenAmount() + " DSM tokens withdrawn successfully!", 
                                Toast.LENGTH_SHORT).show();
                    } else {
                        Toast.makeText(VaultActivity.this, 
                                "Failed to withdraw tokens", 
                                Toast.LENGTH_SHORT).show();
                    }
                })
                .setNegativeButton("Cancel", null)
                .show();
    }
    
    public void onGetDirectionsClick(View view) {
        // Launch navigation to the vault location
        Uri gmmIntentUri = Uri.parse("google.navigation:q=" + 
                vault.getLatitude() + "," + vault.getLongitude());
        Intent mapIntent = new Intent(Intent.ACTION_VIEW, gmmIntentUri);
        mapIntent.setPackage("com.google.android.apps.maps");
        
        if (mapIntent.resolveActivity(getPackageManager()) != null) {
            startActivity(mapIntent);
        } else {
            // If Google Maps is not installed, open with the default map app
            Uri locationUri = Uri.parse("geo:" + vault.getLatitude() + 
                    "," + vault.getLongitude() + "?q=" + vault.getLatitude() + 
                    "," + vault.getLongitude() + "(" + vault.getName() + ")");
            Intent intent = new Intent(Intent.ACTION_VIEW, locationUri);
            
            if (intent.resolveActivity(getPackageManager()) != null) {
                startActivity(intent);
            } else {
                Toast.makeText(this, "No map application available", 
                        Toast.LENGTH_SHORT).show();
            }
        }
    }
    
    public void goBack(View view) {
        finish();
    }
}