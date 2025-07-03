package dsm.vaulthunter.controller;

import android.content.Intent;
import android.os.Bundle;
import android.view.View;
import android.widget.Button;
import android.widget.TextView;
import android.widget.Toast;

import androidx.appcompat.app.AppCompatActivity;
import androidx.cardview.widget.CardView;

import dsm.vaulthunter.model.CharacterType;
import dsm.vaulthunter.util.ImageScaler;
import teste.lucasvegi.pokemongooffline.R;

/**
 * Activity for selecting a character during registration
 */
public class CharacterSelectionActivity extends AppCompatActivity {
    private static final String TAG = "CharacterSelectionActivity";
    
    private CharacterType selectedCharacter = null;
    private TextView characterDescription;
    private Button selectButton;
    
    private CardView robotCard;
    private CardView maleCard;
    private CardView skullCard;
    private CardView femaleCard;
    
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_character_selection);
        
        // Initialize views
        characterDescription = findViewById(R.id.characterDescription);
        selectButton = findViewById(R.id.selectButton);
        
        // Initialize cards
        robotCard = findViewById(R.id.robotCharacterCard);
        maleCard = findViewById(R.id.maleCharacterCard);
        skullCard = findViewById(R.id.skullCharacterCard);
        femaleCard = findViewById(R.id.blondeCharacterCard);
        
        // Disable select button initially
        selectButton.setEnabled(false);
        selectButton.setAlpha(0.5f);
        
        // Auto-scale character images
        autoScaleCharacterImages();
    }
    
    /**
     * Auto-scale character images to fit their containers
     */
    private void autoScaleCharacterImages() {
        // Use the ImageScaler utility to auto-size the character images
        // For this to work properly, the ImageViews should already have their dimensions set in the layout
        
        // For now, we'll use post() to schedule the image loading for when the views have been measured
        View content = findViewById(android.R.id.content);
        content.post(new Runnable() {
            @Override
            public void run() {
                // Load each character image
                if (findViewById(R.id.robotImage) != null) {
                    ImageScaler.loadImageIntoView(findViewById(R.id.robotImage), R.drawable.character_robot);
                }
                
                if (findViewById(R.id.maleImage) != null) {
                    ImageScaler.loadImageIntoView(findViewById(R.id.maleImage), R.drawable.character_male);
                }
                
                if (findViewById(R.id.skullImage) != null) {
                    ImageScaler.loadImageIntoView(findViewById(R.id.skullImage), R.drawable.character_skull);
                }
                
                if (findViewById(R.id.blondeImage) != null) {
                    ImageScaler.loadImageIntoView(findViewById(R.id.blondeImage), R.drawable.character_female);
                }
            }
        });
    }
    
    /**
     * Handle character selection
     * @param view The clicked view
     */
    public void onCharacterSelected(View view) {
        // Reset all card backgrounds
        resetCardSelections();
        
        // Set the selected character based on the tag
        String tag = (String) view.getTag();
        if (tag != null) {
            try {
                selectedCharacter = CharacterType.valueOf(tag);
                
                // Highlight the selected card
                view.setBackgroundResource(R.color.dsm_primary_transparent);
                
                // Update description
                characterDescription.setText(selectedCharacter.getDetailedDescription());
                
                // Enable select button
                selectButton.setEnabled(true);
                selectButton.setAlpha(1.0f);
                
                // Toast for feedback
                Toast.makeText(this, 
                        selectedCharacter.getName() + " selected", 
                        Toast.LENGTH_SHORT).show();
            } catch (IllegalArgumentException e) {
                // Invalid tag
                selectedCharacter = null;
                characterDescription.setText("Select a character to see their description");
                selectButton.setEnabled(false);
                selectButton.setAlpha(0.5f);
            }
        }
    }
    
    /**
     * Reset all card selection highlights
     */
    private void resetCardSelections() {
        robotCard.setCardBackgroundColor(getResources().getColor(R.color.dsm_surface));
        maleCard.setCardBackgroundColor(getResources().getColor(R.color.dsm_surface));
        skullCard.setCardBackgroundColor(getResources().getColor(R.color.dsm_surface));
        femaleCard.setCardBackgroundColor(getResources().getColor(R.color.dsm_surface));
    }
    
    /**
     * Confirm character selection
     * @param view The clicked view
     */
    public void onSelectCharacter(View view) {
        if (selectedCharacter != null) {
            // Return the selected character to the registration activity
            Intent intent = new Intent();
            intent.putExtra("characterType", selectedCharacter.name());
            setResult(RESULT_OK, intent);
            finish();
        } else {
            Toast.makeText(this, "Please select a character first", Toast.LENGTH_SHORT).show();
        }
    }
    
    /**
     * Handle back button click
     * @param view The clicked view
     */
    public void goBack(View view) {
        // Just finish the activity, no result
        setResult(RESULT_CANCELED);
        finish();
    }
    
    @Override
    public void onBackPressed() {
        // Handle back button press same as click
        setResult(RESULT_CANCELED);
        super.onBackPressed();
    }
}