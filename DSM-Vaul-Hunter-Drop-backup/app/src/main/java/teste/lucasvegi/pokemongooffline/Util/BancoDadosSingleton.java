package teste.lucasvegi.pokemongooffline.Util;

import android.content.ContentValues;
import android.content.Context;
import android.database.Cursor;
import android.database.sqlite.SQLiteDatabase;
import android.database.sqlite.SQLiteOpenHelper;

import java.text.ParseException;
import java.text.SimpleDateFormat;
import java.util.ArrayList;
import java.util.Date;

import teste.lucasvegi.pokemongooffline.Model.Aparecimento;
import teste.lucasvegi.pokemongooffline.Model.ControladoraFachadaSingleton;
import teste.lucasvegi.pokemongooffline.Model.Doce;
import teste.lucasvegi.pokemongooffline.Model.Pokemon;
import teste.lucasvegi.pokemongooffline.Model.PokemonCapturado;
import teste.lucasvegi.pokemongooffline.Model.Tipo;
import teste.lucasvegi.pokemongooffline.Model.Usuario;

/**
 * Classe que implementa o padrão Singleton para fornecer um objeto único de acesso
 * ao banco de dados SQLite da aplicação.
 */
public class BancoDadosSingleton extends SQLiteOpenHelper {
    private static BancoDadosSingleton INSTANCIA = null;
    private static final String DATABASE_NAME = "pokemonGoOfflineDB";
    private static final int DATABASE_VERSION = 1;
    private static SimpleDateFormat sdf = new SimpleDateFormat("yyyy-MM-dd HH:mm:ss");
    
    // Tabela de Usuários
    private static final String TABELA_USUARIO = "usuario";
    private static final String USUARIO_LOGIN = "login";
    private static final String USUARIO_NOME = "nome";
    private static final String USUARIO_SENHA = "senha";
    private static final String USUARIO_SEXO = "sexo";
    private static final String USUARIO_PONTOS = "pontos";
    private static final String USUARIO_NIVEL = "nivel";
    
    // Tabela de Tipos
    private static final String TABELA_TIPO = "tipo";
    private static final String TIPO_ID = "id";
    private static final String TIPO_NOME = "nome";
    
    // Tabela de Pokémon
    private static final String TABELA_POKEMON = "pokemon";
    private static final String POKEMON_NUMERO = "numero";
    private static final String POKEMON_NOME = "nome";
    private static final String POKEMON_DESCRICAO = "descricao";
    private static final String POKEMON_CP_BASE = "cp_base";
    private static final String POKEMON_HP_BASE = "hp_base";
    private static final String POKEMON_PESO = "peso";
    private static final String POKEMON_ALTURA = "altura";
    private static final String POKEMON_RARIDADE = "raridade";
    private static final String POKEMON_TIPO_PRIMARIO = "tipo_primario";
    private static final String POKEMON_TIPO_SECUNDARIO = "tipo_secundario";
    private static final String POKEMON_DOCE = "doce";
    private static final String POKEMON_DISTANCIA = "distancia";
    private static final String POKEMON_CANDY_EVOLUCAO = "candy_evolucao";
    private static final String POKEMON_EVOLUCAO = "evolucao";
    
    // Tabela de Doces
    private static final String TABELA_DOCE = "doce";
    private static final String DOCE_ID = "id";
    private static final String DOCE_NOME = "nome";
    
    // Tabela de Pokémon Capturados
    private static final String TABELA_CAPTURADOS = "pokemon_capturado";
    private static final String CAPTURADO_ID = "id";
    private static final String CAPTURADO_POKEMON = "pokemon";
    private static final String CAPTURADO_USUARIO = "usuario";
    private static final String CAPTURADO_CP = "cp";
    private static final String CAPTURADO_HP = "hp";
    private static final String CAPTURADO_PESO = "peso";
    private static final String CAPTURADO_ALTURA = "altura";
    private static final String CAPTURADO_DATA_CAPTURA = "data_captura";
    private static final String CAPTURADO_LATITUDE = "latitude";
    private static final String CAPTURADO_LONGITUDE = "longitude";
    
    // Tabela de Inventário de Doces
    private static final String TABELA_INVENTARIO_DOCE = "inventario_doce";
    private static final String INVENTARIO_DOCE_USUARIO = "usuario";
    private static final String INVENTARIO_DOCE_DOCE = "doce";
    private static final String INVENTARIO_DOCE_QUANTIDADE = "quantidade";
    
    // Tabela de Aparecimento de Pokémon
    private static final String TABELA_APARECIMENTO = "aparecimento";
    private static final String APARECIMENTO_ID = "id";
    private static final String APARECIMENTO_POKEMON = "pokemon";
    private static final String APARECIMENTO_LATITUDE = "latitude";
    private static final String APARECIMENTO_LONGITUDE = "longitude";
    private static final String APARECIMENTO_DATA_HORA = "data_hora";
    private static final String APARECIMENTO_ATIVO = "ativo";

    // Construtor privado (Singleton)
    private BancoDadosSingleton(Context context) {
        super(context, DATABASE_NAME, null, DATABASE_VERSION);
    }

    /**
     * Método que retorna a instância única do banco de dados (Singleton)
     * @param context Contexto da aplicação
     * @return Instância única do banco de dados
     */
    public static synchronized BancoDadosSingleton getInstancia(Context context) {
        if (INSTANCIA == null) {
            INSTANCIA = new BancoDadosSingleton(context.getApplicationContext());
        }
        return INSTANCIA;
    }

    @Override
    public void onCreate(SQLiteDatabase db) {
        // Criação da tabela de usuários
        String criarTabelaUsuario = "CREATE TABLE " + TABELA_USUARIO + " ("
                + USUARIO_LOGIN + " TEXT PRIMARY KEY, "
                + USUARIO_NOME + " TEXT NOT NULL, "
                + USUARIO_SENHA + " TEXT NOT NULL, "
                + USUARIO_SEXO + " TEXT CHECK (sexo IN ('M', 'F')), "
                + USUARIO_PONTOS + " INTEGER DEFAULT 0, "
                + USUARIO_NIVEL + " INTEGER DEFAULT 1);";
        
        // Criação da tabela de tipos
        String criarTabelaTipo = "CREATE TABLE " + TABELA_TIPO + " ("
                + TIPO_ID + " INTEGER PRIMARY KEY AUTOINCREMENT, "
                + TIPO_NOME + " TEXT UNIQUE NOT NULL);";
        
        // Criação da tabela de doces
        String criarTabelaDoce = "CREATE TABLE " + TABELA_DOCE + " ("
                + DOCE_ID + " INTEGER PRIMARY KEY AUTOINCREMENT, "
                + DOCE_NOME + " TEXT UNIQUE NOT NULL);";
        
        // Criação da tabela de Pokémon
        String criarTabelaPokemon = "CREATE TABLE " + TABELA_POKEMON + " ("
                + POKEMON_NUMERO + " INTEGER PRIMARY KEY, "
                + POKEMON_NOME + " TEXT UNIQUE NOT NULL, "
                + POKEMON_DESCRICAO + " TEXT, "
                + POKEMON_CP_BASE + " INTEGER NOT NULL, "
                + POKEMON_HP_BASE + " INTEGER NOT NULL, "
                + POKEMON_PESO + " REAL NOT NULL, "
                + POKEMON_ALTURA + " REAL NOT NULL, "
                + POKEMON_RARIDADE + " INTEGER NOT NULL, "
                + POKEMON_TIPO_PRIMARIO + " INTEGER NOT NULL, "
                + POKEMON_TIPO_SECUNDARIO + " INTEGER, "
                + POKEMON_DOCE + " INTEGER NOT NULL, "
                + POKEMON_DISTANCIA + " REAL NOT NULL, "
                + POKEMON_CANDY_EVOLUCAO + " INTEGER, "
                + POKEMON_EVOLUCAO + " INTEGER, "
                + "FOREIGN KEY (" + POKEMON_TIPO_PRIMARIO + ") REFERENCES " + TABELA_TIPO + "(" + TIPO_ID + "), "
                + "FOREIGN KEY (" + POKEMON_TIPO_SECUNDARIO + ") REFERENCES " + TABELA_TIPO + "(" + TIPO_ID + "), "
                + "FOREIGN KEY (" + POKEMON_DOCE + ") REFERENCES " + TABELA_DOCE + "(" + DOCE_ID + "), "
                + "FOREIGN KEY (" + POKEMON_EVOLUCAO + ") REFERENCES " + TABELA_POKEMON + "(" + POKEMON_NUMERO + "));";
        
        // Criação da tabela de Pokémon capturados
        String criarTabelaCapturados = "CREATE TABLE " + TABELA_CAPTURADOS + " ("
                + CAPTURADO_ID + " INTEGER PRIMARY KEY AUTOINCREMENT, "
                + CAPTURADO_POKEMON + " INTEGER NOT NULL, "
                + CAPTURADO_USUARIO + " TEXT NOT NULL, "
                + CAPTURADO_CP + " INTEGER NOT NULL, "
                + CAPTURADO_HP + " INTEGER NOT NULL, "
                + CAPTURADO_PESO + " REAL NOT NULL, "
                + CAPTURADO_ALTURA + " REAL NOT NULL, "
                + CAPTURADO_DATA_CAPTURA + " TEXT NOT NULL, "
                + CAPTURADO_LATITUDE + " REAL NOT NULL, "
                + CAPTURADO_LONGITUDE + " REAL NOT NULL, "
                + "FOREIGN KEY (" + CAPTURADO_POKEMON + ") REFERENCES " + TABELA_POKEMON + "(" + POKEMON_NUMERO + "), "
                + "FOREIGN KEY (" + CAPTURADO_USUARIO + ") REFERENCES " + TABELA_USUARIO + "(" + USUARIO_LOGIN + "));";
        
        // Criação da tabela de inventário de doces
        String criarTabelaInventarioDoce = "CREATE TABLE " + TABELA_INVENTARIO_DOCE + " ("
                + INVENTARIO_DOCE_USUARIO + " TEXT NOT NULL, "
                + INVENTARIO_DOCE_DOCE + " INTEGER NOT NULL, "
                + INVENTARIO_DOCE_QUANTIDADE + " INTEGER NOT NULL, "
                + "PRIMARY KEY (" + INVENTARIO_DOCE_USUARIO + ", " + INVENTARIO_DOCE_DOCE + "), "
                + "FOREIGN KEY (" + INVENTARIO_DOCE_USUARIO + ") REFERENCES " + TABELA_USUARIO + "(" + USUARIO_LOGIN + "), "
                + "FOREIGN KEY (" + INVENTARIO_DOCE_DOCE + ") REFERENCES " + TABELA_DOCE + "(" + DOCE_ID + "));";
        
        // Criação da tabela de aparecimento de Pokémon
        String criarTabelaAparecimento = "CREATE TABLE " + TABELA_APARECIMENTO + " ("
                + APARECIMENTO_ID + " INTEGER PRIMARY KEY AUTOINCREMENT, "
                + APARECIMENTO_POKEMON + " INTEGER NOT NULL, "
                + APARECIMENTO_LATITUDE + " REAL NOT NULL, "
                + APARECIMENTO_LONGITUDE + " REAL NOT NULL, "
                + APARECIMENTO_DATA_HORA + " TEXT NOT NULL, "
                + APARECIMENTO_ATIVO + " INTEGER NOT NULL DEFAULT 1, "
                + "FOREIGN KEY (" + APARECIMENTO_POKEMON + ") REFERENCES " + TABELA_POKEMON + "(" + POKEMON_NUMERO + "));";
        
        // Executando os scripts de criação das tabelas
        db.execSQL(criarTabelaUsuario);
        db.execSQL(criarTabelaTipo);
        db.execSQL(criarTabelaDoce);
        db.execSQL(criarTabelaPokemon);
        db.execSQL(criarTabelaCapturados);
        db.execSQL(criarTabelaInventarioDoce);
        db.execSQL(criarTabelaAparecimento);
        
        // Povoando o banco de dados com dados iniciais
        povoarBancoDados(db);
    }

    @Override
    public void onUpgrade(SQLiteDatabase db, int oldVersion, int newVersion) {
        // Na primeira versão, simplesmente apagamos e recriamos as tabelas
        db.execSQL("DROP TABLE IF EXISTS " + TABELA_INVENTARIO_DOCE);
        db.execSQL("DROP TABLE IF EXISTS " + TABELA_CAPTURADOS);
        db.execSQL("DROP TABLE IF EXISTS " + TABELA_APARECIMENTO);
        db.execSQL("DROP TABLE IF EXISTS " + TABELA_POKEMON);
        db.execSQL("DROP TABLE IF EXISTS " + TABELA_TIPO);
        db.execSQL("DROP TABLE IF EXISTS " + TABELA_DOCE);
        db.execSQL("DROP TABLE IF EXISTS " + TABELA_USUARIO);
        
        onCreate(db);
    }

    /**
     * Popula o banco de dados com dados iniciais (tipos, doces e pokémons)
     */
    private void povoarBancoDados(SQLiteDatabase db) {
        // Inserindo tipos de Pokémon
        inserirTipo(db, "Normal");
        inserirTipo(db, "Fire");
        inserirTipo(db, "Water");
        inserirTipo(db, "Electric");
        inserirTipo(db, "Grass");
        inserirTipo(db, "Ice");
        inserirTipo(db, "Fighting");
        inserirTipo(db, "Poison");
        inserirTipo(db, "Ground");
        inserirTipo(db, "Flying");
        inserirTipo(db, "Psychic");
        inserirTipo(db, "Bug");
        inserirTipo(db, "Rock");
        inserirTipo(db, "Ghost");
        inserirTipo(db, "Dragon");
        inserirTipo(db, "Dark");
        inserirTipo(db, "Steel");
        inserirTipo(db, "Fairy");
        
        // Inserindo doces de Pokémon
        inserirDoce(db, "Bulbasaur Candy");
        inserirDoce(db, "Charmander Candy");
        inserirDoce(db, "Squirtle Candy");
        inserirDoce(db, "Caterpie Candy");
        inserirDoce(db, "Weedle Candy");
        inserirDoce(db, "Pidgey Candy");
        inserirDoce(db, "Rattata Candy");
        inserirDoce(db, "Spearow Candy");
        inserirDoce(db, "Ekans Candy");
        inserirDoce(db, "Pikachu Candy");
        
        // Inserindo alguns Pokémon básicos
        // Formato: número, nome, descrição, CP base, HP base, peso, altura, raridade, tipo1, tipo2, doceId, distância, candyEvolução, evolução
        
        // Bulbasaur
        inserirPokemon(db, 1, "Bulbasaur", 
                "A strange seed was planted on its back at birth. The plant sprouts and grows with this Pokémon.", 
                600, 90, 6.9, 0.7, 2, 5, 8, 1, 3.0, 25, 2);
        
        // Ivysaur
        inserirPokemon(db, 2, "Ivysaur", 
                "When the bulb on its back grows large, it appears to lose the ability to stand on its hind legs.", 
                1200, 120, 13.0, 1.0, 3, 5, 8, 1, 3.0, 100, 3);
        
        // Venusaur
        inserirPokemon(db, 3, "Venusaur", 
                "The plant blooms when it is absorbing solar energy. It stays on the move to seek sunlight.", 
                2400, 160, 100.0, 2.0, 4, 5, 8, 1, 3.0, null, null);
        
        // Charmander
        inserirPokemon(db, 4, "Charmander", 
                "Obviously prefers hot places. When it rains, steam is said to spout from the tip of its tail.", 
                600, 78, 8.5, 0.6, 2, 2, null, 2, 3.0, 25, 5);
        
        // Charmeleon
        inserirPokemon(db, 5, "Charmeleon", 
                "When it swings its burning tail, it elevates the temperature to unbearably high levels.", 
                1100, 116, 19.0, 1.1, 3, 2, null, 2, 3.0, 100, 6);
        
        // Charizard
        inserirPokemon(db, 6, "Charizard", 
                "Spits fire that is hot enough to melt boulders. Known to cause forest fires unintentionally.", 
                2100, 156, 90.5, 1.7, 4, 2, 10, 2, 3.0, null, null);
        
        // Squirtle
        inserirPokemon(db, 7, "Squirtle", 
                "After birth, its back swells and hardens into a shell. Powerfully sprays foam from its mouth.", 
                500, 88, 9.0, 0.5, 2, 3, null, 3, 3.0, 25, 8);
        
        // Wartortle
        inserirPokemon(db, 8, "Wartortle", 
                "Often hides in water to stalk unwary prey. For swimming fast, it moves its ears to maintain balance.", 
                1000, 118, 22.5, 1.0, 3, 3, null, 3, 3.0, 100, 9);
        
        // Blastoise
        inserirPokemon(db, 9, "Blastoise", 
                "A brutal Pokémon with pressurized water jets on its shell. They are used for high-speed tackles.", 
                2000, 158, 85.5, 1.6, 4, 3, null, 3, 3.0, null, null);
        
        // Pikachu
        inserirPokemon(db, 25, "Pikachu", 
                "When several of these Pokémon gather, their electricity can build and cause lightning storms.", 
                650, 82, 6.0, 0.4, 3, 4, null, 10, 1.0, 50, 26);
        
        // Raichu
        inserirPokemon(db, 26, "Raichu", 
                "Its long tail serves as a ground to protect itself from its own high-voltage power.", 
                1650, 122, 30.0, 0.8, 4, 4, null, 10, 1.0, null, null);
    }

    /**
     * Insere um tipo de Pokémon no banco de dados
     */
    private long inserirTipo(SQLiteDatabase db, String nome) {
        ContentValues valores = new ContentValues();
        valores.put(TIPO_NOME, nome);
        return db.insert(TABELA_TIPO, null, valores);
    }

    /**
     * Insere um doce de Pokémon no banco de dados
     */
    private long inserirDoce(SQLiteDatabase db, String nome) {
        ContentValues valores = new ContentValues();
        valores.put(DOCE_NOME, nome);
        return db.insert(TABELA_DOCE, null, valores);
    }

    /**
     * Insere um Pokémon no banco de dados
     */
    private long inserirPokemon(SQLiteDatabase db, int numero, String nome, String descricao,
                              int cpBase, int hpBase, double peso, double altura, int raridade,
                              int tipoPrimario, Integer tipoSecundario, int doce, double distancia,
                              Integer candyEvolucao, Integer evolucao) {
        ContentValues valores = new ContentValues();
        valores.put(POKEMON_NUMERO, numero);
        valores.put(POKEMON_NOME, nome);
        valores.put(POKEMON_DESCRICAO, descricao);
        valores.put(POKEMON_CP_BASE, cpBase);
        valores.put(POKEMON_HP_BASE, hpBase);
        valores.put(POKEMON_PESO, peso);
        valores.put(POKEMON_ALTURA, altura);
        valores.put(POKEMON_RARIDADE, raridade);
        valores.put(POKEMON_TIPO_PRIMARIO, tipoPrimario);
        
        if (tipoSecundario != null) {
            valores.put(POKEMON_TIPO_SECUNDARIO, tipoSecundario);
        }
        
        valores.put(POKEMON_DOCE, doce);
        valores.put(POKEMON_DISTANCIA, distancia);
        
        if (candyEvolucao != null) {
            valores.put(POKEMON_CANDY_EVOLUCAO, candyEvolucao);
        }
        
        if (evolucao != null) {
            valores.put(POKEMON_EVOLUCAO, evolucao);
        }
        
        return db.insert(TABELA_POKEMON, null, valores);
    }

    /**
     * Verifica se um usuário existe no banco de dados
     */
    public boolean verificarUsuario(String login, String senha) {
        SQLiteDatabase db = this.getReadableDatabase();
        
        String query = "SELECT * FROM " + TABELA_USUARIO +
                " WHERE " + USUARIO_LOGIN + " = ? AND " + USUARIO_SENHA + " = ?";
        
        Cursor cursor = db.rawQuery(query, new String[]{login, senha});
        boolean existe = cursor.getCount() > 0;
        
        cursor.close();
        return existe;
    }

    /**
     * Insere um novo usuário no banco de dados
     */
    public long inserirUsuario(Usuario usuario) {
        SQLiteDatabase db = this.getWritableDatabase();
        
        ContentValues valores = new ContentValues();
        valores.put(USUARIO_LOGIN, usuario.getLogin());
        valores.put(USUARIO_NOME, usuario.getNome());
        valores.put(USUARIO_SENHA, usuario.getSenha());
        valores.put(USUARIO_SEXO, usuario.getSexo());
        valores.put(USUARIO_PONTOS, usuario.getPontos());
        valores.put(USUARIO_NIVEL, usuario.getNivel());
        
        return db.insert(TABELA_USUARIO, null, valores);
    }

    /**
     * Busca um usuário pelo login
     */
    public Usuario buscarUsuario(String login) {
        SQLiteDatabase db = this.getReadableDatabase();
        
        String query = "SELECT * FROM " + TABELA_USUARIO +
                " WHERE " + USUARIO_LOGIN + " = ?";
        
        Cursor cursor = db.rawQuery(query, new String[]{login});
        
        Usuario usuario = null;
        
        if (cursor.moveToFirst()) {
            usuario = new Usuario(
                    cursor.getString(cursor.getColumnIndex(USUARIO_LOGIN)),
                    cursor.getString(cursor.getColumnIndex(USUARIO_NOME)),
                    cursor.getString(cursor.getColumnIndex(USUARIO_SENHA)),
                    cursor.getString(cursor.getColumnIndex(USUARIO_SEXO)).charAt(0),
                    cursor.getInt(cursor.getColumnIndex(USUARIO_PONTOS)),
                    cursor.getInt(cursor.getColumnIndex(USUARIO_NIVEL))
            );
        }
        
        cursor.close();
        return usuario;
    }

    /**
     * Busca todos os Pokémon do banco de dados
     */
    public ArrayList<Pokemon> buscarTodosPokemon() {
        SQLiteDatabase db = this.getReadableDatabase();
        ArrayList<Pokemon> pokemons = new ArrayList<>();
        
        String query = "SELECT * FROM " + TABELA_POKEMON + " ORDER BY " + POKEMON_NUMERO;
        
        Cursor cursor = db.rawQuery(query, null);
        
        if (cursor.moveToFirst()) {
            do {
                // Buscando os tipos do Pokémon
                int idTipoPrimario = cursor.getInt(cursor.getColumnIndex(POKEMON_TIPO_PRIMARIO));
                Tipo tipoPrimario = buscarTipo(idTipoPrimario);
                
                Tipo tipoSecundario = null;
                if (!cursor.isNull(cursor.getColumnIndex(POKEMON_TIPO_SECUNDARIO))) {
                    int idTipoSecundario = cursor.getInt(cursor.getColumnIndex(POKEMON_TIPO_SECUNDARIO));
                    tipoSecundario = buscarTipo(idTipoSecundario);
                }
                
                // Buscando o doce do Pokémon
                int idDoce = cursor.getInt(cursor.getColumnIndex(POKEMON_DOCE));
                Doce doce = buscarDoce(idDoce);
                
                // Buscando a evolução do Pokémon (se houver)
                Pokemon evolucao = null;
                if (!cursor.isNull(cursor.getColumnIndex(POKEMON_EVOLUCAO))) {
                    int numeroEvolucao = cursor.getInt(cursor.getColumnIndex(POKEMON_EVOLUCAO));
                    evolucao = buscarPokemon(numeroEvolucao);
                }
                
                // Criando o objeto Pokémon
                Pokemon pokemon = new Pokemon(
                        cursor.getInt(cursor.getColumnIndex(POKEMON_NUMERO)),
                        cursor.getString(cursor.getColumnIndex(POKEMON_NOME)),
                        cursor.getString(cursor.getColumnIndex(POKEMON_DESCRICAO)),
                        cursor.getInt(cursor.getColumnIndex(POKEMON_CP_BASE)),
                        cursor.getInt(cursor.getColumnIndex(POKEMON_HP_BASE)),
                        cursor.getDouble(cursor.getColumnIndex(POKEMON_PESO)),
                        cursor.getDouble(cursor.getColumnIndex(POKEMON_ALTURA)),
                        cursor.getInt(cursor.getColumnIndex(POKEMON_RARIDADE)),
                        tipoPrimario,
                        tipoSecundario,
                        doce,
                        cursor.getDouble(cursor.getColumnIndex(POKEMON_DISTANCIA))
                );
                
                // Configurando a evolução e o candy necessário
                if (!cursor.isNull(cursor.getColumnIndex(POKEMON_CANDY_EVOLUCAO))) {
                    pokemon.setCandyEvolucao(cursor.getInt(cursor.getColumnIndex(POKEMON_CANDY_EVOLUCAO)));
                }
                
                if (evolucao != null) {
                    pokemon.setEvolucao(evolucao);
                }
                
                pokemons.add(pokemon);
            } while (cursor.moveToNext());
        }
        
        cursor.close();
        return pokemons;
    }

    /**
     * Busca um Pokémon pelo número
     */
    public Pokemon buscarPokemon(int numero) {
        SQLiteDatabase db = this.getReadableDatabase();
        
        String query = "SELECT * FROM " + TABELA_POKEMON +
                " WHERE " + POKEMON_NUMERO + " = ?";
        
        Cursor cursor = db.rawQuery(query, new String[]{String.valueOf(numero)});
        
        Pokemon pokemon = null;
        
        if (cursor.moveToFirst()) {
            // Buscando os tipos do Pokémon
            int idTipoPrimario = cursor.getInt(cursor.getColumnIndex(POKEMON_TIPO_PRIMARIO));
            Tipo tipoPrimario = buscarTipo(idTipoPrimario);
            
            Tipo tipoSecundario = null;
            if (!cursor.isNull(cursor.getColumnIndex(POKEMON_TIPO_SECUNDARIO))) {
                int idTipoSecundario = cursor.getInt(cursor.getColumnIndex(POKEMON_TIPO_SECUNDARIO));
                tipoSecundario = buscarTipo(idTipoSecundario);
            }
            
            // Buscando o doce do Pokémon
            int idDoce = cursor.getInt(cursor.getColumnIndex(POKEMON_DOCE));
            Doce doce = buscarDoce(idDoce);
            
            // Criando o objeto Pokémon (sem a evolução para evitar recursão infinita)
            pokemon = new Pokemon(
                    cursor.getInt(cursor.getColumnIndex(POKEMON_NUMERO)),
                    cursor.getString(cursor.getColumnIndex(POKEMON_NOME)),
                    cursor.getString(cursor.getColumnIndex(POKEMON_DESCRICAO)),
                    cursor.getInt(cursor.getColumnIndex(POKEMON_CP_BASE)),
                    cursor.getInt(cursor.getColumnIndex(POKEMON_HP_BASE)),
                    cursor.getDouble(cursor.getColumnIndex(POKEMON_PESO)),
                    cursor.getDouble(cursor.getColumnIndex(POKEMON_ALTURA)),
                    cursor.getInt(cursor.getColumnIndex(POKEMON_RARIDADE)),
                    tipoPrimario,
                    tipoSecundario,
                    doce,
                    cursor.getDouble(cursor.getColumnIndex(POKEMON_DISTANCIA))
            );
            
            // Configurando o candy necessário para evolução
            if (!cursor.isNull(cursor.getColumnIndex(POKEMON_CANDY_EVOLUCAO))) {
                pokemon.setCandyEvolucao(cursor.getInt(cursor.getColumnIndex(POKEMON_CANDY_EVOLUCAO)));
            }
        }
        
        cursor.close();
        
        // Se o Pokémon foi encontrado, tentamos buscar sua evolução
        if (pokemon != null) {
            // Buscando novamente para configurar a evolução
            String queryEvolucao = "SELECT " + POKEMON_EVOLUCAO + " FROM " + TABELA_POKEMON +
                    " WHERE " + POKEMON_NUMERO + " = ?";
            
            Cursor cursorEvolucao = db.rawQuery(queryEvolucao, new String[]{String.valueOf(numero)});
            
            if (cursorEvolucao.moveToFirst() && !cursorEvolucao.isNull(0)) {
                int numeroEvolucao = cursorEvolucao.getInt(0);
                
                // Verificamos se não estamos em um ciclo (evolução aponta para o próprio Pokémon)
                if (numeroEvolucao != numero) {
                    Pokemon evolucao = buscarPokemon(numeroEvolucao);
                    pokemon.setEvolucao(evolucao);
                }
            }
            
            cursorEvolucao.close();
        }
        
        return pokemon;
    }

    /**
     * Busca um tipo de Pokémon pelo ID
     */
    private Tipo buscarTipo(int id) {
        SQLiteDatabase db = this.getReadableDatabase();
        
        String query = "SELECT * FROM " + TABELA_TIPO +
                " WHERE " + TIPO_ID + " = ?";
        
        Cursor cursor = db.rawQuery(query, new String[]{String.valueOf(id)});
        
        Tipo tipo = null;
        
        if (cursor.moveToFirst()) {
            tipo = new Tipo(
                    cursor.getInt(cursor.getColumnIndex(TIPO_ID)),
                    cursor.getString(cursor.getColumnIndex(TIPO_NOME))
            );
        }
        
        cursor.close();
        return tipo;
    }

    /**
     * Busca um doce de Pokémon pelo ID
     */
    private Doce buscarDoce(int id) {
        SQLiteDatabase db = this.getReadableDatabase();
        
        String query = "SELECT * FROM " + TABELA_DOCE +
                " WHERE " + DOCE_ID + " = ?";
        
        Cursor cursor = db.rawQuery(query, new String[]{String.valueOf(id)});
        
        Doce doce = null;
        
        if (cursor.moveToFirst()) {
            doce = new Doce(
                    cursor.getInt(cursor.getColumnIndex(DOCE_ID)),
                    cursor.getString(cursor.getColumnIndex(DOCE_NOME))
            );
        }
        
        cursor.close();
        return doce;
    }

    /**
     * Insere um novo aparecimento de Pokémon no mapa
     */
    public long inserirAparecimento(Aparecimento aparecimento) {
        SQLiteDatabase db = this.getWritableDatabase();
        
        ContentValues valores = new ContentValues();
        valores.put(APARECIMENTO_POKEMON, aparecimento.getPokemon().getNumero());
        valores.put(APARECIMENTO_LATITUDE, aparecimento.getLatitude());
        valores.put(APARECIMENTO_LONGITUDE, aparecimento.getLongitude());
        valores.put(APARECIMENTO_DATA_HORA, sdf.format(aparecimento.getDataHora()));
        valores.put(APARECIMENTO_ATIVO, aparecimento.isAtivo() ? 1 : 0);
        
        return db.insert(TABELA_APARECIMENTO, null, valores);
    }

    /**
     * Busca todos os aparecimentos ativos no mapa
     */
    public ArrayList<Aparecimento> buscarAparecimentosAtivos() {
        SQLiteDatabase db = this.getReadableDatabase();
        ArrayList<Aparecimento> aparecimentos = new ArrayList<>();
        
        String query = "SELECT * FROM " + TABELA_APARECIMENTO +
                " WHERE " + APARECIMENTO_ATIVO + " = 1";
        
        Cursor cursor = db.rawQuery(query, null);
        
        if (cursor.moveToFirst()) {
            do {
                // Buscando o Pokémon do aparecimento
                int numeroPokemon = cursor.getInt(cursor.getColumnIndex(APARECIMENTO_POKEMON));
                Pokemon pokemon = buscarPokemon(numeroPokemon);
                
                // Convertendo a data
                Date dataHora = null;
                try {
                    dataHora = sdf.parse(cursor.getString(cursor.getColumnIndex(APARECIMENTO_DATA_HORA)));
                } catch (ParseException e) {
                    e.printStackTrace();
                    dataHora = new Date(); // Data atual como fallback
                }
                
                // Criando o objeto Aparecimento
                Aparecimento aparecimento = new Aparecimento(
                        cursor.getInt(cursor.getColumnIndex(APARECIMENTO_ID)),
                        pokemon,
                        cursor.getDouble(cursor.getColumnIndex(APARECIMENTO_LATITUDE)),
                        cursor.getDouble(cursor.getColumnIndex(APARECIMENTO_LONGITUDE)),
                        dataHora,
                        cursor.getInt(cursor.getColumnIndex(APARECIMENTO_ATIVO)) == 1
                );
                
                aparecimentos.add(aparecimento);
            } while (cursor.moveToNext());
        }
        
        cursor.close();
        return aparecimentos;
    }

    /**
     * Desativa um aparecimento (quando um Pokémon é capturado)
     */
    public boolean desativarAparecimento(int id) {
        SQLiteDatabase db = this.getWritableDatabase();
        
        ContentValues valores = new ContentValues();
        valores.put(APARECIMENTO_ATIVO, 0);
        
        return db.update(TABELA_APARECIMENTO, valores, APARECIMENTO_ID + " = ?",
                new String[]{String.valueOf(id)}) > 0;
    }

    /**
     * Registra a captura de um Pokémon
     */
    public long registrarCaptura(PokemonCapturado pokemonCapturado) {
        SQLiteDatabase db = this.getWritableDatabase();
        
        ContentValues valores = new ContentValues();
        valores.put(CAPTURADO_POKEMON, pokemonCapturado.getPokemon().getNumero());
        valores.put(CAPTURADO_USUARIO, pokemonCapturado.getUsuario().getLogin());
        valores.put(CAPTURADO_CP, pokemonCapturado.getCp());
        valores.put(CAPTURADO_HP, pokemonCapturado.getHp());
        valores.put(CAPTURADO_PESO, pokemonCapturado.getPeso());
        valores.put(CAPTURADO_ALTURA, pokemonCapturado.getAltura());
        valores.put(CAPTURADO_DATA_CAPTURA, sdf.format(pokemonCapturado.getDataCaptura()));
        valores.put(CAPTURADO_LATITUDE, pokemonCapturado.getLatitude());
        valores.put(CAPTURADO_LONGITUDE, pokemonCapturado.getLongitude());
        
        long id = db.insert(TABELA_CAPTURADOS, null, valores);
        
        // Atualizando a quantidade de doces do usuário
        if (id != -1) {
            atualizarDoces(pokemonCapturado.getUsuario().getLogin(), 
                    pokemonCapturado.getPokemon().getDoce().getId(), 3); // 3 doces por captura
        }
        
        return id;
    }

    /**
     * Atualiza a quantidade de doces de um usuário
     */
    private boolean atualizarDoces(String login, int idDoce, int quantidade) {
        SQLiteDatabase db = this.getWritableDatabase();
        
        // Verificando se o usuário já tem esse tipo de doce
        String query = "SELECT * FROM " + TABELA_INVENTARIO_DOCE +
                " WHERE " + INVENTARIO_DOCE_USUARIO + " = ? AND " + INVENTARIO_DOCE_DOCE + " = ?";
        
        Cursor cursor = db.rawQuery(query, new String[]{login, String.valueOf(idDoce)});
        
        boolean resultado = false;
        
        if (cursor.moveToFirst()) {
            // Usuário já tem esse doce, atualizamos a quantidade
            int quantidadeAtual = cursor.getInt(cursor.getColumnIndex(INVENTARIO_DOCE_QUANTIDADE));
            int novaQuantidade = quantidadeAtual + quantidade;
            
            ContentValues valores = new ContentValues();
            valores.put(INVENTARIO_DOCE_QUANTIDADE, novaQuantidade);
            
            resultado = db.update(TABELA_INVENTARIO_DOCE, valores, 
                    INVENTARIO_DOCE_USUARIO + " = ? AND " + INVENTARIO_DOCE_DOCE + " = ?",
                    new String[]{login, String.valueOf(idDoce)}) > 0;
        } else {
            // Usuário não tem esse doce, inserimos
            ContentValues valores = new ContentValues();
            valores.put(INVENTARIO_DOCE_USUARIO, login);
            valores.put(INVENTARIO_DOCE_DOCE, idDoce);
            valores.put(INVENTARIO_DOCE_QUANTIDADE, quantidade);
            
            resultado = db.insert(TABELA_INVENTARIO_DOCE, null, valores) != -1;
        }
        
        cursor.close();
        return resultado;
    }

    /**
     * Busca a quantidade de doces que um usuário tem de um determinado tipo
     */
    public int buscarQuantidadeDoces(String login, int idDoce) {
        SQLiteDatabase db = this.getReadableDatabase();
        
        String query = "SELECT " + INVENTARIO_DOCE_QUANTIDADE + " FROM " + TABELA_INVENTARIO_DOCE +
                " WHERE " + INVENTARIO_DOCE_USUARIO + " = ? AND " + INVENTARIO_DOCE_DOCE + " = ?";
        
        Cursor cursor = db.rawQuery(query, new String[]{login, String.valueOf(idDoce)});
        
        int quantidade = 0;
        
        if (cursor.moveToFirst()) {
            quantidade = cursor.getInt(0);
        }
        
        cursor.close();
        return quantidade;
    }

    /**
     * Busca todos os Pokémon capturados por um usuário
     */
    public ArrayList<PokemonCapturado> buscarPokemonCapturados(String login) {
        SQLiteDatabase db = this.getReadableDatabase();
        ArrayList<PokemonCapturado> capturados = new ArrayList<>();
        
        String query = "SELECT * FROM " + TABELA_CAPTURADOS +
                " WHERE " + CAPTURADO_USUARIO + " = ? ORDER BY " + CAPTURADO_DATA_CAPTURA + " DESC";
        
        Cursor cursor = db.rawQuery(query, new String[]{login});
        
        if (cursor.moveToFirst()) {
            Usuario usuario = buscarUsuario(login);
            
            do {
                // Buscando o Pokémon
                int numeroPokemon = cursor.getInt(cursor.getColumnIndex(CAPTURADO_POKEMON));
                Pokemon pokemon = buscarPokemon(numeroPokemon);
                
                // Convertendo a data
                Date dataCaptura = null;
                try {
                    dataCaptura = sdf.parse(cursor.getString(cursor.getColumnIndex(CAPTURADO_DATA_CAPTURA)));
                } catch (ParseException e) {
                    e.printStackTrace();
                    dataCaptura = new Date(); // Data atual como fallback
                }
                
                // Criando o objeto PokemonCapturado
                PokemonCapturado pokemonCapturado = new PokemonCapturado(
                        cursor.getInt(cursor.getColumnIndex(CAPTURADO_ID)),
                        pokemon,
                        usuario,
                        cursor.getInt(cursor.getColumnIndex(CAPTURADO_CP)),
                        cursor.getInt(cursor.getColumnIndex(CAPTURADO_HP)),
                        cursor.getDouble(cursor.getColumnIndex(CAPTURADO_PESO)),
                        cursor.getDouble(cursor.getColumnIndex(CAPTURADO_ALTURA)),
                        dataCaptura,
                        cursor.getDouble(cursor.getColumnIndex(CAPTURADO_LATITUDE)),
                        cursor.getDouble(cursor.getColumnIndex(CAPTURADO_LONGITUDE))
                );
                
                capturados.add(pokemonCapturado);
            } while (cursor.moveToNext());
        }
        
        cursor.close();
        return capturados;
    }

    /**
     * Atualiza os pontos e o nível de um usuário
     */
    public boolean atualizarUsuario(Usuario usuario) {
        SQLiteDatabase db = this.getWritableDatabase();
        
        ContentValues valores = new ContentValues();
        valores.put(USUARIO_PONTOS, usuario.getPontos());
        valores.put(USUARIO_NIVEL, usuario.getNivel());
        
        return db.update(TABELA_USUARIO, valores, USUARIO_LOGIN + " = ?",
                new String[]{usuario.getLogin()}) > 0;
    }
}
