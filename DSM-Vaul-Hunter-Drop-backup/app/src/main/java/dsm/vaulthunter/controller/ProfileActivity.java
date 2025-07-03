package dsm.vaulthunter.controller;

import android.content.Intent;
import android.graphics.Color;
import android.os.Bundle;
import android.view.Gravity;
import android.view.View;
import android.widget.ImageView;
import android.widget.LinearLayout;
import android.widget.ProgressBar;
import android.widget.TextView;
import android.widget.Toast;

import androidx.appcompat.app.AlertDialog;
import androidx.appcompat.app.AppCompatActivity;

import java.util.List;
import java.util.Map;

import dsm.vaulthunter.model.AppController;
import dsm.vaulthunter.model.Crystal;
import dsm.vaulthunter.model.DSMClient;
import dsm.vaulthunter.model.Treasure;
import dsm.vaulthunter.model.User;
import dsm.vaulthunter.model.Vault;
import teste.lucasvegi.pokemongooffline.R;

/**
 * Activity to display and manage user profile
 */
public class ProfileActivity extends AppCompatActivity {
    private static final String TAG = "ProfileActivity";
    
    private User user;
    
    private ImageView characterImage;
    private TextView userName;
    private TextView userLevel;
    private ProgressBar experienceBar;
    private TextView experienceText;
    private TextView tokenBalance;
    private TextView treasuresFound;
    private TextView uniqueTreasures;
    private TextView vaultsOpened;
    private LinearLayout crystalContainer;
    
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_profile);
        
        // Get current user
        user = AppController.getInstance().getLoggedInUser();
        
        // Initialize views
        initializeViews();
        
        // Update UI with user data
        updateUI();
    }
    
    @Override
    protected void onResume() {
        super.onResume();
        
        // Refresh user data in case it was updated elsewhere
        user = AppController.getInstance().getLoggedInUser();
        updateUI();
    }
    
    private void initializeViews() {
        characterImage = findViewById(R.id.characterImage);
        userName = findViewById(R.id.userName);
        userLevel = findViewById(R.id.userLevel);
        experienceBar = findViewById(R.id.experienceBar);
        experienceText = findViewById(R.id.experienceText);
        tokenBalance = findViewById(R.id.tokenBalance);
        treasuresFound = findViewById(R.id.treasuresFound);
        uniqueTreasures = findViewById(R.id.uniqueTreasures);
        vaultsOpened = findViewById(R.id.vaultsOpened);
        crystalContainer = findViewById(R.id.crystalContainer);
    }
    
    private void updateUI() {
        // Set character image based on character type
        int characterImageRes = getResources().getIdentifier(
                "character_" + user.getCharacterType().name().toLowerCase(), 
                "drawable", 
                getPackageName());
        
        if (characterImageRes != 0) {
            characterImage.setImageResource(characterImageRes);
        }
        
        // Set user name and level
        userName.setText(user.getName());
        userLevel.setText(String.valueOf(user.getLevel()));
        
        // Calculate experience for next level
        int currentXP = user.getExperiencePoints();
        int xpForCurrentLevel = 100 * (user.getLevel() - 1) * (user.getLevel() - 1);
        int xpForNextLevel = 100 * user.getLevel() * user.getLevel();
        int xpProgress = currentXP - xpForCurrentLevel;
        int xpNeeded = xpForNextLevel - xpForCurrentLevel;
        
        // Update experience bar and text
        experienceBar.setMax(xpNeeded);
        experienceBar.setProgress(xpProgress);
        experienceText.setText(xpProgress + " / " + xpNeeded + " XP to Level " + (user.getLevel() + 1));
        
        // Set token balance
        tokenBalance.setText(String.valueOf(user.getAvailableTokens()));
        
        // Update collection stats
        int totalTreasures = user.getCollectedTreasures().size();
        treasuresFound.setText(String.valueOf(totalTreasures));
        
        // Count unique treasures
        int uniqueCount = 0;
        for (Treasure treasure : AppController.getInstance().getAllTreasures()) {
            if (user.getTreasureCount(treasure) > 0) {
                uniqueCount++;
            }
        }
        uniqueTreasures.setText(String.valueOf(uniqueCount));
        
        // Count opened vaults
        int vaultCount = 0;
        List<Vault> userVaults = AppController.getInstance().getUserVaults(user);
        if (userVaults != null) {
            for (Vault vault : userVaults) {
                if (vault.getStatus() == Vault.VaultStatus.CLAIMED || 
                    vault.getStatus() == Vault.VaultStatus.WITHDRAWN) {
                    vaultCount++;
                }
            }
        }
        vaultsOpened.setText(String.valueOf(vaultCount));
        
        // Update crystal inventory
        updateCrystalInventory();
    }
    
    private void updateCrystalInventory() {
        // Clear existing crystal views
        crystalContainer.removeAllViews();
        
        // Get user crystals
        Map<Integer, Crystal> userCrystals = user.getCrystals();
        
        if (userCrystals.isEmpty()) {
            TextView emptyText = new TextView(this);
            emptyText.setText("No crystals found yet");
            emptyText.setGravity(Gravity.CENTER);
            emptyText.setPadding(0, 16, 0, 16);
            crystalContainer.addView(emptyText);
            return;
        }
        
        // Add each crystal to the container
        for (Crystal crystal : userCrystals.values()) {
            if (crystal.getQuantity() > 0) {
                addCrystalView(crystal);
            }
        }
    }
    
    private void addCrystalView(Crystal crystal) {
        // Create horizontal layout for the crystal
        LinearLayout crystalLayout = new LinearLayout(this);
        crystalLayout.setOrientation(LinearLayout.HORIZONTAL);
        crystalLayout.setGravity(Gravity.CENTER_VERTICAL);
        crystalLayout.setPadding(0, 8, 0, 8);
        
        // Create crystal icon
        ImageView crystalIcon = new ImageView(this);
        int iconResourceId = getResources().getIdentifier(
                "crystal_" + crystal.getName().toLowerCase(), 
                "drawable", 
                getPackageName());
        
        if (iconResourceId != 0) {
            crystalIcon.setImageResource(iconResourceId);
        } else {
            // Fallback to a default icon
            crystalIcon.setImageResource(R.mipmap.ic_launcher);
        }
        
        // Set image size
        LinearLayout.LayoutParams iconParams = new LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.WRAP_CONTENT,
                LinearLayout.LayoutParams.WRAP_CONTENT
        );
        iconParams.width = 48;
        iconParams.height = 48;
        iconParams.rightMargin = 16;
        crystalIcon.setLayoutParams(iconParams);
        
        // Create crystal name text
        TextView nameText = new TextView(this);
        nameText.setText(crystal.getName() + " Crystal");
        nameText.setTextSize(16);
        nameText.setTextColor(Color.BLACK);
        
        // Set text layout
        LinearLayout.LayoutParams nameParams = new LinearLayout.LayoutParams(
                0,
                LinearLayout.LayoutParams.WRAP_CONTENT,
                1.0f
        );
        nameText.setLayoutParams(nameParams);
        
        // Create quantity text
        TextView quantityText = new TextView(this);
        quantityText.setText("x" + crystal.getQuantity());
        quantityText.setTextSize(16);
        quantityText.setTextColor(Color.BLACK);
        quantityText.setGravity(Gravity.END);
        
        // Add views to layout
        crystalLayout.addView(crystalIcon);
        crystalLayout.addView(nameText);
        crystalLayout.addView(quantityText);
        
        // Add to container
        crystalContainer.addView(crystalLayout);
        
        // Add divider except for last item
        if (crystal.getId() != user.getCrystals().size()) {
            View divider = new View(this);
            divider.setBackgroundColor(Color.LTGRAY);
            LinearLayout.LayoutParams dividerParams = new LinearLayout.LayoutParams(
                    LinearLayout.LayoutParams.MATCH_PARENT,
                    1
            );
            dividerParams.topMargin = 8;
            dividerParams.bottomMargin = 8;
            divider.setLayoutParams(dividerParams);
            crystalContainer.addView(divider);
        }
    }
    
    public void onWithdrawClick(View view) {
        if (!DSMClient.getInstance().isConnected()) {
            Toast.makeText(this, "Not connected to DSM network", Toast.LENGTH_SHORT).show();
            return;
        }
        
        final long availableTokens = user.getAvailableTokens();
        
        if (availableTokens <= 0) {
            Toast.makeText(this, "No tokens available to withdraw", Toast.LENGTH_SHORT).show();
            return;
        }
        
        // Show confirmation dialog
        new AlertDialog.Builder(this)
                .setTitle("Withdraw Tokens")
                .setMessage("Are you sure you want to withdraw " + availableTokens + " DSM tokens to your wallet?")
                .setPositiveButton("Withdraw", (dialog, which) -> {
                    // Simulate token withdrawal
                    user.setTokensWithdrawn(user.getTokensWithdrawn() + availableTokens);
                    
                    // Save to database
                    AppController.getInstance().updateUser(user);
                    
                    // Update UI
                    updateUI();
                    
                    Toast.makeText(ProfileActivity.this, 
                            availableTokens + " DSM tokens withdrawn successfully", 
                            Toast.LENGTH_SHORT).show();
                })
                .setNegativeButton("Cancel", null)
                .show();
    }
    
    public void onViewTreasuresClick(View view) {
        startActivity(new Intent(this, TreasureDexActivity.class));
    }
    
    public void goBack(View view) {
        finish();
    }
}