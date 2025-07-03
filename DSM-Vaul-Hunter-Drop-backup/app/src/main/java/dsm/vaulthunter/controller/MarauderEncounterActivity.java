package dsm.vaulthunter.controller;

import android.os.Bundle;
import android.os.Handler;
import android.view.View;
import android.view.animation.Animation;
import android.view.animation.AnimationUtils;
import android.widget.Button;
import android.widget.ImageView;
import android.widget.TextView;
import android.widget.Toast;

import androidx.appcompat.app.AppCompatActivity;

import java.util.Random;

import dsm.vaulthunter.model.AppController;
import dsm.vaulthunter.model.Crystal;
import dsm.vaulthunter.model.Marauder;
import dsm.vaulthunter.model.User;
import dsm.vaulthunter.util.CutsceneManager;
import dsm.vaulthunter.util.RandomUtil;
import teste.lucasvegi.pokemongooffline.R;

/**
 * Activity for handling marauder encounters
 */
public class MarauderEncounterActivity extends AppCompatActivity {
    private static final String TAG = "MarauderEncounterActivity";
    
    // Player and marauder data
    private User user;
    private Marauder marauder;
    private boolean isFirstEncounter = false;
    
    // UI elements
    private ImageView marauderImage;
    private TextView marauderName;
    private TextView marauderDescription;
    private Button fightButton;
    private Button fleeButton;
    private TextView resultText;
    
    // Handler for animations
    private Handler handler;
    
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        // Set theme to DSM theme
        setTheme(R.style.DSMTheme);
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_marauder_encounter);
        
        // Get the current user
        user = AppController.getInstance().getLoggedInUser();
        
        // Initialize UI elements
        marauderImage = findViewById(R.id.marauderImage);
        marauderName = findViewById(R.id.marauderName);
        marauderDescription = findViewById(R.id.marauderDescription);
        fightButton = findViewById(R.id.fightButton);
        fleeButton = findViewById(R.id.fleeButton);
        resultText = findViewById(R.id.resultText);
        
        // Initialize handler
        handler = new Handler();
        
        // Check if this is the first encounter
        CutsceneManager cutsceneManager = CutsceneManager.getInstance(this);
        isFirstEncounter = cutsceneManager.shouldPlayCutscene(CutsceneManager.CUTSCENE_MARAUDER_FIRST);
        
        // Generate a random marauder
        generateMarauder();
        
        // Update UI with marauder info
        updateUI();
        
        // Play intro animation
        playIntroAnimation();
        
        // Play cutscene if this is the first encounter
        if (isFirstEncounter && cutsceneManager.cutsceneExists(CutsceneManager.CUTSCENE_MARAUDER_FIRST)) {
            handler.postDelayed(new Runnable() {
                @Override
                public void run() {
                    cutsceneManager.playCutscene(CutsceneManager.CUTSCENE_MARAUDER_FIRST);
                }
            }, 1000); // Show cutscene after a short delay
        }
    }
    
    /**
     * Generate a random marauder
     */
    private void generateMarauder() {
        // Generate random marauder
        String[] names = {"Zephyr", "Vex", "Krator", "Sable", "Drift", "Raze", "Scrap", "Jinx"};
        String[] types = {"Rogue", "Bruiser", "Hunter", "Technician", "Scout"};
        
        String name = names[RandomUtil.getRandomInt(0, names.length - 1)];
        String type = types[RandomUtil.getRandomInt(0, types.length - 1)];
        int level = Math.max(1, user.getLevel() - 1 + RandomUtil.getRandomInt(-1, 1));
        int strength = level * 10 + RandomUtil.getRandomInt(0, 10);
        
        // Create marauder
        marauder = new Marauder(name, type, level, strength);
    }
    
    /**
     * Update UI with marauder info
     */
    private void updateUI() {
        // Set marauder name and type
        marauderName.setText(marauder.getName() + " the " + marauder.getType());
        
        // Set description
        String description = "Level " + marauder.getLevel() + " Marauder\n";
        description += "Strength: " + marauder.getStrength() + "\n\n";
        description += "A dangerous " + marauder.getType().toLowerCase() + " looking to steal your treasures!";
        marauderDescription.setText(description);
        
        // Set marauder image based on type
        int imageResource = getMarauderImageResource(marauder.getType());
        marauderImage.setImageResource(imageResource);
    }
    
    /**
     * Get the image resource for a marauder type
     * @param type Marauder type
     * @return Image resource ID
     */
    private int getMarauderImageResource(String type) {
        // Get image resource based on marauder type
        // For now, just return the default image
        // In a real implementation, you'd have different images for each type
        return R.drawable.marauder_default;
    }
    
    /**
     * Play intro animation for the marauder
     */
    private void playIntroAnimation() {
        // Apply animation to marauder image
        Animation slideIn = AnimationUtils.loadAnimation(this, android.R.anim.slide_in_left);
        slideIn.setDuration(1000);
        marauderImage.startAnimation(slideIn);
        
        // Apply fade-in to text
        Animation fadeIn = AnimationUtils.loadAnimation(this, android.R.anim.fade_in);
        fadeIn.setDuration(1000);
        marauderName.startAnimation(fadeIn);
        marauderDescription.startAnimation(fadeIn);
    }
    
    /**
     * Handle fight button click
     * @param view The clicked view
     */
    public void onFightClick(View view) {
        // Disable buttons
        fightButton.setEnabled(false);
        fleeButton.setEnabled(false);
        
        // Calculate fight outcome
        int userStrength = user.getLevel() * 10;
        int marauderStrength = marauder.getStrength();
        
        // Add some randomness
        userStrength += RandomUtil.getRandomInt(0, 15);
        marauderStrength += RandomUtil.getRandomInt(0, 15);
        
        final boolean userWins = userStrength >= marauderStrength;
        
        // Show animation effect for fight
        Animation shake = AnimationUtils.loadAnimation(this, android.R.anim.shake);
        marauderImage.startAnimation(shake);
        
        // Delay to show animation
        handler.postDelayed(new Runnable() {
            @Override
            public void run() {
                if (userWins) {
                    handleVictory();
                } else {
                    handleDefeat();
                }
            }
        }, 1000);
    }
    
    /**
     * Handle victory against the marauder
     */
    private void handleVictory() {
        // Show victory message
        resultText.setText("Victory! You defeated the marauder.");
        resultText.setVisibility(View.VISIBLE);
        
        // Add experience points
        int xpGained = marauder.getLevel() * 5;
        boolean leveledUp = user.addExperiencePoints(xpGained);
        
        // Show toast with rewards
        String message = "You gained " + xpGained + " XP";
        if (leveledUp) {
            message += " and leveled up to " + user.getLevel() + "!";
        }
        
        // Check for crystal reward
        if (RandomUtil.getRandomInt(1, 100) <= 30) { // 30% chance
            // Give random crystal
            Crystal crystal = getRandomCrystal();
            if (crystal != null) {
                int amount = RandomUtil.getRandomInt(1, 3);
                
                // Add to user's inventory
                Crystal userCrystal = user.getCrystals().get(crystal.getId());
                if (userCrystal == null) {
                    userCrystal = new Crystal(crystal.getId(), crystal.getName(), crystal.getType(), crystal.getIconResource());
                    user.getCrystals().put(crystal.getId(), userCrystal);
                }
                userCrystal.addQuantity(amount);
                
                message += "\nYou found " + amount + " " + crystal.getName() + " Crystal" + (amount > 1 ? "s" : "") + "!";
            }
        }
        
        // Save user changes
        AppController.getInstance().updateUser(user);
        
        Toast.makeText(this, message, Toast.LENGTH_LONG).show();
        
        // Exit after delay
        handler.postDelayed(new Runnable() {
            @Override
            public void run() {
                finish();
            }
        }, 3000);
    }
    
    /**
     * Handle defeat by the marauder
     */
    private void handleDefeat() {
        // Show defeat message
        resultText.setText("Defeat! The marauder was too strong.");
        resultText.setVisibility(View.VISIBLE);
        
        // Lose a random crystal if user has any
        if (!user.getCrystals().isEmpty() && RandomUtil.getRandomInt(1, 100) <= 50) { // 50% chance
            // Get a random crystal from user's inventory
            Object[] crystalKeys = user.getCrystals().keySet().toArray();
            if (crystalKeys.length > 0) {
                int randomIndex = RandomUtil.getRandomInt(0, crystalKeys.length - 1);
                Integer crystalId = (Integer) crystalKeys[randomIndex];
                Crystal crystal = user.getCrystals().get(crystalId);
                
                // Lose 1 crystal
                if (crystal != null && crystal.getQuantity() > 0) {
                    crystal.useQuantity(1);
                    String message = "The marauder stole 1 " + crystal.getName() + " Crystal!";
                    Toast.makeText(this, message, Toast.LENGTH_LONG).show();
                }
            }
        } else {
            Toast.makeText(this, "You managed to escape without losing anything!", Toast.LENGTH_LONG).show();
        }
        
        // Save user changes
        AppController.getInstance().updateUser(user);
        
        // Exit after delay
        handler.postDelayed(new Runnable() {
            @Override
            public void run() {
                finish();
            }
        }, 3000);
    }
    
    /**
     * Handle flee button click
     * @param view The clicked view
     */
    public void onFleeClick(View view) {
        // Disable buttons
        fightButton.setEnabled(false);
        fleeButton.setEnabled(false);
        
        // Calculate flee success chance (higher level = better chance)
        int fleeChance = 50 + (user.getLevel() * 5);
        boolean fleeSuccess = RandomUtil.getRandomInt(1, 100) <= fleeChance;
        
        if (fleeSuccess) {
            // Show success message
            resultText.setText("You managed to escape!");
            resultText.setVisibility(View.VISIBLE);
            
            // Exit after delay
            handler.postDelayed(new Runnable() {
                @Override
                public void run() {
                    finish();
                }
            }, 2000);
        } else {
            // Failed to flee, force fight
            resultText.setText("Failed to escape! You must fight!");
            resultText.setVisibility(View.VISIBLE);
            
            // Force fight after delay
            handler.postDelayed(new Runnable() {
                @Override
                public void run() {
                    onFightClick(null);
                }
            }, 1500);
        }
    }
    
    /**
     * Get a random crystal
     * @return Random crystal
     */
    private Crystal getRandomCrystal() {
        // Get all crystal types
        List<Crystal> crystalTypes = AppController.getInstance().getCrystalTypes();
        
        if (crystalTypes.isEmpty()) {
            return null;
        }
        
        // Select random crystal
        int index = RandomUtil.getRandomInt(0, crystalTypes.size() - 1);
        return crystalTypes.get(index);
    }
    
    @Override
    protected void onDestroy() {
        super.onDestroy();
        
        // Remove callbacks to prevent leaks
        if (handler != null) {
            handler.removeCallbacksAndMessages(null);
        }
    }
}