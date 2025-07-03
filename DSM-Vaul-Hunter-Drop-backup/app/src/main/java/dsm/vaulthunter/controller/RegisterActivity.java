package dsm.vaulthunter.controller;

import android.content.Intent;
import android.os.Bundle;
import android.text.TextUtils;
import android.view.View;
import android.widget.Button;
import android.widget.EditText;
import android.widget.Toast;

import androidx.appcompat.app.AppCompatActivity;

import dsm.vaulthunter.util.DatabaseHelper;
import teste.lucasvegi.pokemongooffline.R;

/**
 * Activity for user registration
 */
public class RegisterActivity extends AppCompatActivity {
    private static final String TAG = "RegisterActivity";
    
    private EditText nameEditText;
    private EditText emailEditText;
    private EditText passwordEditText;
    private EditText confirmPasswordEditText;
    private Button registerButton;
    private Button cancelButton;
    
    private DatabaseHelper dbHelper;
    
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_register);
        
        // Initialize UI elements
        initializeViews();
        
        // Initialize database helper
        dbHelper = DatabaseHelper.getInstance(this);
        
        // Set up click listeners
        registerButton.setOnClickListener(this::onRegisterClick);
        cancelButton.setOnClickListener(this::onCancelClick);
    }
    
    private void initializeViews() {
        nameEditText = findViewById(R.id.editTextName);
        emailEditText = findViewById(R.id.editTextEmail);
        passwordEditText = findViewById(R.id.editTextPassword);
        confirmPasswordEditText = findViewById(R.id.editTextConfirmPassword);
        registerButton = findViewById(R.id.buttonRegister);
        cancelButton = findViewById(R.id.buttonCancel);
    }
    
    private void onRegisterClick(View view) {
        // Get user input
        String name = nameEditText.getText().toString().trim();
        String email = emailEditText.getText().toString().trim();
        String password = passwordEditText.getText().toString().trim();
        String confirmPassword = confirmPasswordEditText.getText().toString().trim();
        
        // Validate input
        if (TextUtils.isEmpty(name)) {
            nameEditText.setError("Name is required");
            return;
        }
        
        if (TextUtils.isEmpty(email)) {
            emailEditText.setError("Email is required");
            return;
        }
        
        if (TextUtils.isEmpty(password)) {
            passwordEditText.setError("Password is required");
            return;
        }
        
        if (!password.equals(confirmPassword)) {
            confirmPasswordEditText.setError("Passwords do not match");
            return;
        }
        
        // Check if user already exists
        if (dbHelper.userExists(email)) {
            emailEditText.setError("Email already registered");
            return;
        }
        
        // Proceed to character selection
        Intent intent = new Intent(this, CharacterSelectionActivity.class);
        intent.putExtra("user_name", name);
        intent.putExtra("user_email", email);
        intent.putExtra("user_password", password);
        startActivity(intent);
    }
    
    private void onCancelClick(View view) {
        // Return to login
        finish();
    }
}