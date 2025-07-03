package dsm.vaulthunter.controller;

import android.os.Bundle;
import android.view.View;
import android.widget.Button;
import android.widget.ImageView;
import android.widget.LinearLayout;
import android.widget.TextView;
import android.widget.Toast;

import androidx.appcompat.app.AppCompatActivity;
import androidx.cardview.widget.CardView;

import java.text.DecimalFormat;

import dsm.vaulthunter.model.AppController;
import dsm.vaulthunter.model.CollectedTreasure;
import dsm.vaulthunter.model.Crystal;
import dsm.vaulthunter.model.Treasure;
import dsm.vaulthunter.model.TreasureType;
import dsm.vaulthunter.model.User;
import teste.lucasvegi.pokemongooffline.R;

/**
 * Activity to display detailed information about a treasure
 */
public class TreasureDetailsActivity extends AppCompatActivity {
    private static final String TAG = "TreasureDetailsActivity";
    
    private User user;
    private CollectedTreasure collectedTreasure;
    private Treasure treasure;
    
    private TextView treasureNumber;
    private TextView treasureName;
    private TextView treasureTypes;
    private LinearLayout typeIcons;
    private TextView powerLevel;
    private TextView durability;
    private TextView weight;
    private TextView size;
    private TextView rarity;
    private TextView treasureDescription;
    private TextView upgradeInfo;
    private TextView crystalRequirement;
    private Button upgradeButton;
    private CardView upgradeCard;
    private ImageView treasureImage;
    
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_treasure_details);
        
        // Get the user and collected treasure from intent
        user = AppController.getInstance().getLoggedInUser();
        int treasurePosition = getIntent().getIntExtra("treasurePosition", -1);
        
        if (treasurePosition == -1 || treasurePosition >= user.getCollectedTreasures().size()) {
            Toast.makeText(this, "Error loading treasure details", Toast.LENGTH_SHORT).show();
            finish();
            return;
        }
        
        collectedTreasure = user.getCollectedTreasures().get(treasurePosition);
        treasure = collectedTreasure.getTreasure();
        
        // Initialize views
        initializeViews();
        
        // Update UI with treasure data
        updateUI();
    }
    
    private void initializeViews() {
        treasureNumber = findViewById(R.id.treasureNumber);
        treasureName = findViewById(R.id.treasureName);
        treasureTypes = findViewById(R.id.treasureTypes);
        typeIcons = findViewById(R.id.typeIcons);
        powerLevel = findViewById(R.id.powerLevel);
        durability = findViewById(R.id.durability);
        weight = findViewById(R.id.weight);
        size = findViewById(R.id.size);
        rarity = findViewById(R.id.rarity);
        treasureDescription = findViewById(R.id.treasureDescription);
        upgradeInfo = findViewById(R.id.upgradeInfo);
        crystalRequirement = findViewById(R.id.crystalRequirement);
        upgradeButton = findViewById(R.id.upgradeButton);
        upgradeCard = findViewById(R.id.upgradeCard);
        treasureImage = findViewById(R.id.treasureImage);
    }
    
    private void updateUI() {
        // Set basic treasure info
        treasureNumber.setText("#" + String.format("%03d", treasure.getNumber()));
        treasureName.setText(treasure.getName());
        treasureTypes.setText(treasure.getTypesString());
        
        // Set treasure image
        int imageResourceId = getResources().getIdentifier(
                "p" + treasure.getNumber(), "drawable", getPackageName());
        
        if (imageResourceId != 0) {
            treasureImage.setImageResource(imageResourceId);
        }
        
        // Add type icons
        addTypeIcons();
        
        // Set treasure stats
        powerLevel.setText(String.valueOf(treasure.getPowerLevel()));
        durability.setText(String.valueOf(treasure.getDurability()));
        
        DecimalFormat df = new DecimalFormat("#.##");
        weight.setText(df.format(treasure.getWeight()) + " kg");
        size.setText(df.format(treasure.getSize()) + " cm");
        
        // Set rarity text
        String rarityText;
        switch (treasure.getRarity()) {
            case 1:
                rarityText = "Common";
                break;
            case 2:
                rarityText = "Uncommon";
                break;
            case 3:
                rarityText = "Rare";
                break;
            case 4:
                rarityText = "Epic";
                break;
            case 5:
                rarityText = "Legendary";
                break;
            default:
                rarityText = "Unknown";
        }
        rarity.setText(rarityText);
        
        // Set description
        treasureDescription.setText(treasure.getDescription());
        
        // Set upgrade info
        updateUpgradeInfo();
    }
    
    private void addTypeIcons() {
        // Clear existing icons
        typeIcons.removeAllViews();
        
        // Add primary type icon
        addTypeIcon(treasure.getPrimaryType());
        
        // Add secondary type icon if present
        if (treasure.getSecondaryType() != null) {
            addTypeIcon(treasure.getSecondaryType());
        }
    }
    
    private void addTypeIcon(TreasureType type) {
        ImageView typeIcon = new ImageView(this);
        
        // Try to load the icon resource
        int iconResourceId = getResources().getIdentifier(
                "type_" + type.getName().toLowerCase(), "drawable", getPackageName());
        
        if (iconResourceId != 0) {
            typeIcon.setImageResource(iconResourceId);
        } else {
            // Fallback to a default icon
            typeIcon.setImageResource(R.mipmap.ic_launcher);
        }
        
        // Set layout parameters
        LinearLayout.LayoutParams params = new LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.WRAP_CONTENT,
                LinearLayout.LayoutParams.WRAP_CONTENT
        );
        params.width = 100;
        params.height = 100;
        params.setMargins(16, 0, 16, 0);
        
        typeIcon.setLayoutParams(params);
        
        // Add to the linear layout
        typeIcons.addView(typeIcon);
    }
    
    private void updateUpgradeInfo() {
        if (treasure.canUpgrade()) {
            // Show upgrade card
            upgradeCard.setVisibility(View.VISIBLE);
            
            // Show upgrade stage
            upgradeInfo.setText("Current stage: " + treasure.getUpgradeStage());
            
            // Show crystal requirement
            Crystal requiredCrystal = treasure.getCrystal();
            int requiredAmount = treasure.getRequiredCrystals();
            
            crystalRequirement.setText("Requires " + requiredAmount + " " +
                    requiredCrystal.getName() + " crystals to upgrade");
            
            // Check if user has enough crystals
            Crystal userCrystal = user.getCrystals().get(requiredCrystal.getId());
            int userAmount = (userCrystal != null) ? userCrystal.getQuantity() : 0;
            
            if (userAmount >= requiredAmount) {
                upgradeButton.setEnabled(true);
                upgradeButton.setText("Upgrade Treasure (" + userAmount + "/" + requiredAmount + ")");
            } else {
                upgradeButton.setEnabled(false);
                upgradeButton.setText("Not Enough Crystals (" + userAmount + "/" + requiredAmount + ")");
            }
        } else {
            // Hide upgrade card for max level treasures
            upgradeCard.setVisibility(View.GONE);
        }
    }
    
    public void onUpgradeClick(View view) {
        if (treasure.canUpgrade()) {
            // Attempt to upgrade the treasure
            if (user.upgradeTreasure(collectedTreasure)) {
                // Update the treasure reference
                treasure = collectedTreasure.getTreasure();
                
                // Update the UI
                updateUI();
                
                // Show success message
                Toast.makeText(this, "Treasure upgraded successfully!", Toast.LENGTH_SHORT).show();
                
                // Save to database
                AppController.getInstance().updateUser(user);
            } else {
                Toast.makeText(this, "Failed to upgrade treasure", Toast.LENGTH_SHORT).show();
            }
        }
    }
    
    public void goBack(View view) {
        finish();
    }
}