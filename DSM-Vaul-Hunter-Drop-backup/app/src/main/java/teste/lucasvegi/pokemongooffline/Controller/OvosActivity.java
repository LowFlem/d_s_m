package teste.lucasvegi.pokemongooffline.Controller;

import android.app.Activity;
import android.content.Intent;
import android.location.Location;
import android.media.MediaPlayer;
import android.os.Bundle;
import android.util.Log;
import android.view.Gravity;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.AdapterView;
import android.widget.ImageView;
import android.widget.ListView;
import android.widget.TextView;
import android.widget.Toast;

import java.util.ArrayList;
import java.util.List;

import teste.lucasvegi.pokemongooffline.Model.ControladoraFachadaSingleton;
import teste.lucasvegi.pokemongooffline.Model.Ovo;
import teste.lucasvegi.pokemongooffline.R;
import teste.lucasvegi.pokemongooffline.View.AdapterOvos;

public class OvosActivity extends Activity implements AdapterView.OnItemClickListener {
    private List<Ovo> ovos;
    private MediaPlayer mediaPlayer;
    private Toast toastChocado;
    private List<Ovo> ovosChocados = new ArrayList<Ovo>();
    private Location localizacaoAtual = null;
    
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_ovos);
        try {
            TextView txtTotal = (TextView) findViewById(R.id.txtOvosTotal);

            ovos = ControladoraFachadaSingleton.getInstance().getOvos();
            ListView listView = (ListView) findViewById(R.id.listaOvos);

            Intent it = getIntent();
            localizacaoAtual = it.getParcelableExtra("location");

            for (int i = 0; i < ovos.size(); i++) {
                if (ovos.get(i).getLocalizacao() == null && ovos.get(i).getIncubado() == 1) {
                    ovos.get(i).setLocalizacao(localizacaoAtual);
                }

                if (ovos.get(i).getIncubado() == 1) {
                    double distancia = localizacaoAtual.distanceTo(ovos.get(i).getLocalizacao()) / 1000;
                    Log.i("OVOS", "Distância: " + distancia);
                    ovos.get(i).setKmAndado(ovos.get(i).getKmAndado() + distancia);
                    ovos.get(i).setLocalizacao(localizacaoAtual);
                    ControladoraFachadaSingleton.getInstance().setKmAndado(ovos.get(i).getIdOvo(), ovos.get(i).getKmAndado());

                    // Check if egg has hatched
                    if (ovos.get(i).getKmAndado() >= ovos.get(i).getKm()) {
                        ovos.get(i).setChocado(1);
                        ControladoraFachadaSingleton.getInstance().setChocado(ovos.get(i).getIdOvo(), 1);

                        // Updated method name from Chocar to hatchEgg
                        ControladoraFachadaSingleton.getInstance().getUsuario().hatchEgg(ovos.get(i).getLocalizacao(), ovos.get(i).getIdOvo());
                        
                        // Mark egg as displayed
                        ControladoraFachadaSingleton.getInstance().setExibido(ovos.get(i).getIdOvo(), 1);

                        // Remove egg from list
                        Ovo o = ovos.get(i);
                        ovosChocados.add(new Ovo(o.getIdOvo(), o.getidPokemon(), o.getIdTipoOvo(), o.getIncubado(), o.getChocado(), o.getExibido(), o.getKmAndado()));
                        ovos.remove(o);
                    }
                }
            }
            // Total eggs in list
            int total = ControladoraFachadaSingleton.getInstance().getOvos().size();
            txtTotal.setText("Eggs: " + total + "/9");

            AdapterOvos adapterOvos = new AdapterOvos(ovos, this);
            listView.setAdapter(adapterOvos);
            listView.setOnItemClickListener(this);
            mediaPlayer = MediaPlayer.create(getBaseContext(), R.raw.tema_menu);
            mediaPlayer.setLooping(true);
            mediaPlayer.start();
            exibirChocados();
        } catch (Exception e) {
            Log.e("EGGS", "ERROR: " + e.getMessage());
        }
    }
    
    @Override
    protected void onDestroy() {
        super.onDestroy();

        try {
            // Stop menu theme music and release resource
            mediaPlayer.pause();
            mediaPlayer.release();
        } catch (Exception e) {
            Log.e("EGGS", "ERROR: " + e.getMessage());
        }
    }

    public void clickVoltar(View v) {
        for (int i = 0; i < ovos.size(); i++) {
            if (ovos.get(i).getLocalizacao() == null && ovos.get(i).getIncubado() == 1) {
                ovos.get(i).setLocalizacao(localizacaoAtual);
            }
        }
        finish();
    }

    public void exibirChocados() {
        Ovo o;
        if (ovosChocados.size() == 1) {
            LayoutInflater inflater = getLayoutInflater();
            final View layout = inflater.inflate(R.layout.toast_pkmn_chocado, (ViewGroup) findViewById(R.id.toast_pkmn_chocado));
            TextView nomePkmnOvo = (TextView) layout.findViewById(R.id.txtPokemon);
            ImageView fotoPkmnOvo = (ImageView) layout.findViewById(R.id.imgPokemon);
            final String nome = ControladoraFachadaSingleton.getInstance().getPokemonOvo(ovosChocados.get(0).getIdOvo()).getNome();
            nomePkmnOvo.setText("Hooray! " + nome + " has hatched!");
            int foto = ControladoraFachadaSingleton.getInstance().getPokemonOvo(ovosChocados.get(0).getIdOvo()).getFoto();
            fotoPkmnOvo.setImageResource(foto);
            toastChocado = new Toast(getApplicationContext());
            toastChocado.setGravity(Gravity.CENTER_VERTICAL, 0, 100);
            toastChocado.setDuration(Toast.LENGTH_LONG);
            toastChocado.setView(layout);
            toastChocado.show();
            o = ovosChocados.get(0);
            ovosChocados.remove(o);
        }
        
        if (ovosChocados.size() == 2) {
            LayoutInflater inflater = getLayoutInflater();
            final View layout = inflater.inflate(R.layout.toast_pkmn_chocado2, (ViewGroup) findViewById(R.id.toast_pkmn_chocado2));
            TextView nomePkmnOvo = (TextView) layout.findViewById(R.id.txtPokemon);
            ImageView fotoPkmnOvo = (ImageView) layout.findViewById(R.id.imgPokemon);
            ImageView fotoPkmnOvo2 = (ImageView) layout.findViewById(R.id.imgPokemon2);
            final String nome = ControladoraFachadaSingleton.getInstance().getPokemonOvo(ovosChocados.get(0).getIdOvo()).getNome();
            final String nome2 = ControladoraFachadaSingleton.getInstance().getPokemonOvo(ovosChocados.get(1).getIdOvo()).getNome();
            nomePkmnOvo.setText("Hooray! " + nome + " and " + nome2 + " have hatched!");
            int foto = ControladoraFachadaSingleton.getInstance().getPokemonOvo(ovosChocados.get(0).getIdOvo()).getFoto();
            int foto2 = ControladoraFachadaSingleton.getInstance().getPokemonOvo(ovosChocados.get(1).getIdOvo()).getFoto();
            fotoPkmnOvo.setImageResource(foto);
            fotoPkmnOvo2.setImageResource(foto2);
            toastChocado = new Toast(getApplicationContext());
            toastChocado.setGravity(Gravity.CENTER_VERTICAL, 0, 100);
            toastChocado.setDuration(Toast.LENGTH_LONG);
            toastChocado.setView(layout);
            toastChocado.show();
            o = ovosChocados.get(0);
            ovosChocados.remove(o);
            o = ovosChocados.get(0);
            ovosChocados.remove(o);
        }
        
        if (ovosChocados.size() == 3) {
            LayoutInflater inflater = getLayoutInflater();
            final View layout = inflater.inflate(R.layout.toast_pkmn_chocado3, (ViewGroup) findViewById(R.id.toast_pkmn_chocado3));
            TextView nomePkmnOvo = (TextView) layout.findViewById(R.id.txtPokemon);
            ImageView fotoPkmnOvo = (ImageView) layout.findViewById(R.id.imgPokemon);
            ImageView fotoPkmnOvo2 = (ImageView) layout.findViewById(R.id.imgPokemon2);
            ImageView fotoPkmnOvo3 = (ImageView) layout.findViewById(R.id.imgPokemon3);
            final String nome = ControladoraFachadaSingleton.getInstance().getPokemonOvo(ovosChocados.get(0).getIdOvo()).getNome();
            final String nome2 = ControladoraFachadaSingleton.getInstance().getPokemonOvo(ovosChocados.get(1).getIdOvo()).getNome();
            final String nome3 = ControladoraFachadaSingleton.getInstance().getPokemonOvo(ovosChocados.get(2).getIdOvo()).getNome();
            nomePkmnOvo.setText("Hooray! " + nome + ", " + nome2 + " and " + nome3 + " have hatched!");
            int foto = ControladoraFachadaSingleton.getInstance().getPokemonOvo(ovosChocados.get(0).getIdOvo()).getFoto();
            int foto2 = ControladoraFachadaSingleton.getInstance().getPokemonOvo(ovosChocados.get(1).getIdOvo()).getFoto();
            int foto3 = ControladoraFachadaSingleton.getInstance().getPokemonOvo(ovosChocados.get(2).getIdOvo()).getFoto();
            fotoPkmnOvo.setImageResource(foto);
            fotoPkmnOvo2.setImageResource(foto2);
            fotoPkmnOvo3.setImageResource(foto3);
            toastChocado = new Toast(getApplicationContext());
            toastChocado.setGravity(Gravity.CENTER_VERTICAL, 0, 100);
            toastChocado.setDuration(Toast.LENGTH_LONG);
            toastChocado.setView(layout);
            toastChocado.show();
            o = ovosChocados.get(0);
            ovosChocados.remove(o);
            o = ovosChocados.get(0);
            ovosChocados.remove(o);
            o = ovosChocados.get(0);
            ovosChocados.remove(o);
        }
    }

    @Override
    public void onItemClick(AdapterView<?> adapterView, View view, int i, long l) {
        // Item click handler
    }
}
