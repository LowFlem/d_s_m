package teste.lucasvegi.pokemongooffline.Controller;

import android.app.Activity;
import android.content.Intent;
import android.os.Bundle;
import android.util.Log;
import android.view.View;
import android.widget.EditText;
import android.widget.Toast;

import teste.lucasvegi.pokemongooffline.Model.ControladoraFachadaSingleton;
import teste.lucasvegi.pokemongooffline.R;

public class LoginActivity extends Activity {


    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        setContentView(R.layout.activity_login);

    }

    public void clickLogin(View v){
        try {
            Log.i("LOGIN", "Authenticating system entry...");

            EditText edtUser = (EditText) findViewById(R.id.edtUsuarioLogin);
            EditText edtPassword = (EditText) findViewById(R.id.edtSenhaLogin);

            //Get user data
            String user = edtUser.getText().toString();
            String password = edtPassword.getText().toString();

            if (ControladoraFachadaSingleton.getInstance().verificarLogin(user, password)) {
                Intent it = new Intent(this, MapActivity.class);
                startActivity(it);
                finish();
            } else {
                Toast.makeText(this, "Invalid username and/or password!", Toast.LENGTH_SHORT).show();
            }
        }catch (Exception e){
            Log.e("LOGIN", "ERROR: " + e.getMessage());
        }

    }

    public void clickRegister(View v){
        Intent it = new Intent(this, CadastrarActivity.class);
        startActivity(it);
        finish();
    }
}