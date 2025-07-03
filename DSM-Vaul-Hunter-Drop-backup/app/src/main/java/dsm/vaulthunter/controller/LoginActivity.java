package dsm.vaulthunter.controller;

import android.content.Intent;
import android.os.Bundle;
import android.text.TextUtils;
import android.view.View;
import android.widget.Button;
import android.widget.CheckBox;
import android.widget.EditText;
import android.widget.Toast;

import androidx.appcompat.app.AppCompatActivity;

import dsm.vaulthunter.model.AppController;
import dsm.vaulthunter.model.User;
import dsm.vaulthunter.util.AppContext;
import dsm.vaulthunter.util.DatabaseHelper;
import teste.lucasvegi.pokemongooffline.R;

/**
 * Login activity for user authentication
 */
public class LoginActivity extends AppCompatActivity {
    private static final String TAG = "LoginActivity";
    
    private EditText emailEditText;
    private EditText passwordEditText;
    private CheckBox rememberCheckBox;
    private Button loginButton;
    private Button registerButton;
    
    private DatabaseHelper dbHelper;
    
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_login);
        
        // Initialize UI elements
        initializeViews();
        
        // Initialize database helper
        dbHelper = DatabaseHelper.getInstance(this);
        
        // Check for saved credentials
        checkSavedCredentials();
        
        // Set up click listeners
        loginButton.setOnClickListener(this::onLoginClick);
        registerButton.setOnClickListener(this::onRegisterClick);
    }
    
    private void initializeViews() {
        emailEditText = findViewById(R.id.editTextEmail);
        passwordEditText = findViewById(R.id.editTextPassword);
        rememberCheckBox = findViewById(R.id.checkBoxRemember);
        loginButton = findViewById(R.id.buttonLogin);
        registerButton = findViewById(R.id.buttonRegister);
    }
    
    private void checkSavedCredentials() {
        AppContext appContext = (AppContext) getApplicationContext();
        
        if (appContext.hasLoginCredentials()) {
            // In a complete implementation, this would fill in the credentials
            // For now, just check the remember checkbox
            rememberCheckBox.setChecked(true);
        }
    }
    
    private void onLoginClick(View view) {
        // Get user input
        String email = emailEditText.getText().toString().trim();
        String password = passwordEditText.getText().toString().trim();
        
        // Validate input
        if (TextUtils.isEmpty(email)) {
            emailEditText.setError("Email is required");
            return;
        }
        
        if (TextUtils.isEmpty(password)) {
            passwordEditText.setError("Password is required");
            return;
        }
        
        // Authenticate user
        User user = dbHelper.authenticateUser(email, password);
        
        if (user != null) {
            // Save credentials if requested
            if (rememberCheckBox.isChecked()) {
                AppContext appContext = (AppContext) getApplicationContext();
                appContext.saveLoginCredentials(email, password);
            }
            
            // Set logged in user
            AppController.getInstance().setLoggedInUser(user);
            
            // Start the main activity
            startMainActivity();
        } else {
            // For development purposes, allow logging in with any credentials
            // In a production app, this would be removed
            Toast.makeText(this, "Development mode: Creating test user", Toast.LENGTH_SHORT).show();
            
            // Create a test user
            User testUser = new User("Test User", email, password, 'M');
            long userId = dbHelper.addUser(testUser);
            
            if (userId > 0) {
                // Set the user ID
                testUser.setId(userId);
                
                // Set logged in user
                AppController.getInstance().setLoggedInUser(testUser);
                
                // Save credentials if requested
                if (rememberCheckBox.isChecked()) {
                    AppContext appContext = (AppContext) getApplicationContext();
                    appContext.saveLoginCredentials(email, password);
                }
                
                // Start the main activity
                startMainActivity();
            } else {
                Toast.makeText(this, "Login failed", Toast.LENGTH_SHORT).show();
            }
        }
    }
    
    private void onRegisterClick(View view) {
        // Start the register activity
        Intent intent = new Intent(this, RegisterActivity.class);
        startActivity(intent);
    }
    
    private void startMainActivity() {
        // For now, go directly to the map
        Intent intent = new Intent(this, MapActivity.class);
        intent.addFlags(Intent.FLAG_ACTIVITY_CLEAR_TOP | Intent.FLAG_ACTIVITY_NEW_TASK);
        startActivity(intent);
        finish();
    }
}