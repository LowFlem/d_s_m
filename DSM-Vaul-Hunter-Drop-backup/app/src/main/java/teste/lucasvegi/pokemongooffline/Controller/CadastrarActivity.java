package teste.lucasvegi.pokemongooffline.Controller;
import android.content.Intent;
import android.os.Bundle;
import android.view.View;
import android.widget.Button;
import android.widget.EditText;
import android.widget.RadioButton;
import android.widget.RadioGroup;
import android.widget.Toast;

import androidx.appcompat.app.AppCompatActivity;

import teste.lucasvegi.pokemongooffline.Model.ControladoraFachadaSingleton;
import teste.lucasvegi.pokemongooffline.Model.Usuario;
import teste.lucasvegi.pokemongooffline.R;

/**
 * Activity responsável pelo cadastro de novos usuários
 */
public class CadastrarActivity extends AppCompatActivity {

    private EditText editTextLogin;
    private EditText editTextNome;
    private EditText editTextSenha;
    private EditText editTextConfirmarSenha;
    private RadioGroup radioGroupSexo;
    private RadioButton radioButtonMasculino;
    private RadioButton radioButtonFeminino;
    private Button buttonCadastrar;
    private Button buttonCancelar;

    private ControladoraFachadaSingleton controladora;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_cadastrar);

        // Inicializa a controladora
        controladora = ControladoraFachadaSingleton.getInstance();

        // Referências às views
        editTextLogin = findViewById(R.id.editTextLogin);
        editTextNome = findViewById(R.id.editTextNome);
        editTextSenha = findViewById(R.id.editTextSenha);
        editTextConfirmarSenha = findViewById(R.id.editTextConfirmarSenha);
        radioGroupSexo = findViewById(R.id.radioGroupSexo);
        radioButtonMasculino = findViewById(R.id.radioButtonMasculino);
        radioButtonFeminino = findViewById(R.id.radioButtonFeminino);
        buttonCadastrar = findViewById(R.id.buttonCadastrar);
        buttonCancelar = findViewById(R.id.buttonCancelar);

        // Configura o botão de cadastrar
        buttonCadastrar.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                cadastrarUsuario();
            }
        });

        // Configura o botão de cancelar
        buttonCancelar.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                finish();
            }
        });
    }

    /**
     * Realiza o cadastro do usuário
     */
    private void cadastrarUsuario() {
        // Obtém os valores dos campos
        String login = editTextLogin.getText().toString().trim();
        String nome = editTextNome.getText().toString().trim();
        String senha = editTextSenha.getText().toString();
        String confirmarSenha = editTextConfirmarSenha.getText().toString();

        // Verifica se todos os campos foram preenchidos
        if (login.isEmpty() || nome.isEmpty() || senha.isEmpty() || confirmarSenha.isEmpty()) {
            Toast.makeText(this, "Preencha todos os campos", Toast.LENGTH_SHORT).show();
            return;
        }

        // Verifica se as senhas são iguais
        if (!senha.equals(confirmarSenha)) {
            Toast.makeText(this, "As senhas não conferem", Toast.LENGTH_SHORT).show();
            return;
        }

        // Verifica se o usuário selecionou um sexo
        if (radioGroupSexo.getCheckedRadioButtonId() == -1) {
            Toast.makeText(this, "Selecione o sexo", Toast.LENGTH_SHORT).show();
            return;
        }

        // Obtém o sexo selecionado
        char sexo = radioButtonMasculino.isChecked() ? 'M' : 'F';

        // Cria o objeto usuário
        Usuario usuario = new Usuario(login, nome, senha, sexo);

        // Tenta cadastrar o usuário
        if (controladora.cadastrarUsuario(usuario)) {
            Toast.makeText(this, "Cadastro realizado com sucesso!", Toast.LENGTH_LONG).show();
            
            // Define o usuário como logado
            controladora.setUsuarioLogado(usuario);
            
            // Redireciona para a tela do mapa
            Intent intent = new Intent(this, MapActivity.class);
            intent.setFlags(Intent.FLAG_ACTIVITY_NEW_TASK | Intent.FLAG_ACTIVITY_CLEAR_TASK);
            startActivity(intent);
            finish();
        } else {
            Toast.makeText(this, "Erro ao cadastrar usuário. Nome de usuário já existe.", Toast.LENGTH_LONG).show();
        }
    }
}
