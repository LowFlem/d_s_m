package teste.lucasvegi.pokemongooffline.Model;

import android.content.Context;

import java.util.ArrayList;

import teste.lucasvegi.pokemongooffline.Util.BancoDadosSingleton;

/**
 * Classe Controladora que implementa o padrão Singleton e Facade
 * para coordenar as operações do sistema
 */
public class ControladoraFachadaSingleton {
    private static ControladoraFachadaSingleton INSTANCIA = null;
    private BancoDadosSingleton bancoDados;
    private Usuario usuarioLogado;
    private ArrayList<Pokemon> pokemons;
    
    // Construtor privado (Singleton)
    private ControladoraFachadaSingleton() {
        this.pokemons = new ArrayList<>();
    }
    
    /**
     * Método que retorna a instância única da controladora (Singleton)
     * @return Instância única da controladora
     */
    public static synchronized ControladoraFachadaSingleton getInstance() {
        if (INSTANCIA == null) {
            INSTANCIA = new ControladoraFachadaSingleton();
        }
        return INSTANCIA;
    }
    
    /**
     * Inicializa o banco de dados, deve ser chamado no início da aplicação
     * @param context Contexto da aplicação
     */
    public void inicializarBancoDados(Context context) {
        this.bancoDados = BancoDadosSingleton.getInstancia(context);
        this.pokemons = bancoDados.buscarTodosPokemon();
    }
    
    /**
     * Verifica as credenciais do usuário para login
     * @param login Login do usuário
     * @param senha Senha do usuário
     * @return true se as credenciais estiverem corretas
     */
    public boolean verificarLogin(String login, String senha) {
        boolean resultado = bancoDados.verificarUsuario(login, senha);
        
        if (resultado) {
            usuarioLogado = bancoDados.buscarUsuario(login);
        }
        
        return resultado;
    }
    
    /**
     * Cadastra um novo usuário
     * @param usuario Objeto Usuario a ser cadastrado
     * @return true se o cadastro foi bem-sucedido
     */
    public boolean cadastrarUsuario(Usuario usuario) {
        long id = bancoDados.inserirUsuario(usuario);
        
        if (id != -1) {
            usuarioLogado = usuario;
            return true;
        }
        
        return false;
    }
    
    /**
     * Verifica se tem uma sessão aberta (usuário logado)
     * @return true se há um usuário logado
     */
    public boolean temSessao() {
        return usuarioLogado != null;
    }
    
    /**
     * Realiza o logout do usuário atual
     */
    public void logout() {
        usuarioLogado = null;
    }
    
    /**
     * Retorna o usuário atualmente logado
     * @return Usuário logado ou null se ninguém estiver logado
     */
    public Usuario getUsuarioLogado() {
        return usuarioLogado;
    }
    
    /**
     * Define o usuário logado
     * @param usuario Usuário a ser definido como logado
     */
    public void setUsuarioLogado(Usuario usuario) {
        this.usuarioLogado = usuario;
    }
    
    /**
     * Lista todos os Pokémon do jogo
     * @return Lista de Pokémon
     */
    public ArrayList<Pokemon> listarPokemons() {
        return pokemons;
    }
    
    /**
     * Busca um Pokémon pelo número
     * @param numero Número do Pokémon na Pokédex
     * @return Pokémon encontrado ou null
     */
    public Pokemon buscarPokemon(int numero) {
        for (Pokemon pokemon : pokemons) {
            if (pokemon.getNumero() == numero) {
                return pokemon;
            }
        }
        
        return bancoDados.buscarPokemon(numero);
    }
    
    /**
     * Lista os aparecimentos ativos de Pokémon no mapa
     * @return Lista de aparecimentos ativos
     */
    public ArrayList<Aparecimento> listarAparecimentosAtivos() {
        return bancoDados.buscarAparecimentosAtivos();
    }
    
    /**
     * Registra um novo aparecimento de Pokémon no mapa
     * @param aparecimento Objeto Aparecimento a ser registrado
     * @return ID do aparecimento registrado ou -1 em caso de erro
     */
    public long registrarAparecimento(Aparecimento aparecimento) {
        return bancoDados.inserirAparecimento(aparecimento);
    }
    
    /**
     * Desativa um aparecimento (quando um Pokémon é capturado)
     * @param idAparecimento ID do aparecimento a ser desativado
     * @return true se o aparecimento foi desativado com sucesso
     */
    public boolean desativarAparecimento(int idAparecimento) {
        return bancoDados.desativarAparecimento(idAparecimento);
    }
    
    /**
     * Registra a captura de um Pokémon
     * @param pokemonCapturado Objeto PokemonCapturado a ser registrado
     * @return ID do Pokémon capturado ou -1 em caso de erro
     */
    public long registrarCaptura(PokemonCapturado pokemonCapturado) {
        return bancoDados.registrarCaptura(pokemonCapturado);
    }
    
    /**
     * Lista todos os Pokémon capturados pelo usuário logado
     * @return Lista de Pokémon capturados
     */
    public ArrayList<PokemonCapturado> listarPokemonCapturados() {
        if (usuarioLogado != null) {
            return bancoDados.buscarPokemonCapturados(usuarioLogado.getLogin());
        }
        
        return new ArrayList<>();
    }
    
    /**
     * Verifica a quantidade de doces que o usuário tem de um determinado tipo
     * @param idDoce ID do doce
     * @return Quantidade de doces
     */
    public int verificarQuantidadeDoces(int idDoce) {
        if (usuarioLogado != null) {
            return bancoDados.buscarQuantidadeDoces(usuarioLogado.getLogin(), idDoce);
        }
        
        return 0;
    }
    
    /**
     * Atualiza os pontos e o nível do usuário logado
     * @return true se a atualização foi bem-sucedida
     */
    public boolean atualizarUsuario(Usuario usuario) {
        return bancoDados.atualizarUsuario(usuario);
    }
    
    /**
     * Evolui um Pokémon capturado para sua próxima forma
     * @param pokemonCapturado Pokémon a ser evoluído
     * @return ID do novo Pokémon evoluído ou -1 em caso de erro
     */
    public long evoluirPokemon(PokemonCapturado pokemonCapturado) {
        // Verifica se o Pokémon pode evoluir
        Pokemon pokemon = pokemonCapturado.getPokemon();
        if (!pokemon.podeEvoluir()) {
            return -1;
        }
        
        // Verifica se o usuário tem doces suficientes
        int quantidadeDoces = verificarQuantidadeDoces(pokemon.getDoce().getId());
        if (quantidadeDoces < pokemon.getCandyEvolucao()) {
            return -1;
        }
        
        // TODO: Implementar a evolução do Pokémon
        // 1. Consumir os doces necessários
        // 2. Desativar o Pokémon atual
        // 3. Criar um novo Pokémon evoluído
        
        return -1; // Ainda não implementado
    }
    
    /**
     * Sorteia um Pokémon aleatório para aparecer no mapa
     * Tem 15% de chance de gerar um baú de tesouro em vez de um Pokémon
     * @return Pokémon sorteado
     */
    public Pokemon sorteiaPokemon() {
        if (pokemons.isEmpty()) {
            // Se a lista ainda não foi carregada, tenta carregar
            pokemons = bancoDados.buscarTodosPokemon();
            
            // Se mesmo assim estiver vazia, retorna null
            if (pokemons.isEmpty()) {
                return null;
            }
        }
        
        // 15% de chance de gerar um baú de tesouro
        if (Math.random() < 0.15) {
            // Para baús, usamos números especiais começando em 900
            // 901 = Bronze, 902 = Silver, 903 = Gold
            double random = Math.random();
            int chestNumber;
            String chestName;
            
            if (random < 0.8) {
                // 80% chance de Bronze (901)
                chestNumber = 901;
                chestName = "Bronze Treasure Chest";
            } else if (random < 0.95) {
                // 15% chance de Silver (902)
                chestNumber = 902;
                chestName = "Silver Treasure Chest";
            } else {
                // 5% chance de Gold (903)
                chestNumber = 903;
                chestName = "Gold Treasure Chest";
            }
            
            // Cria um Pokémon especial para representar o baú
            Pokemon chest = new Pokemon(
                chestNumber,
                chestName,
                "A Treasure Chest filled with valuable DSM tokens",
                500, // CP
                100, // HP
                10.0, // Peso
                0.8, // Altura
                5, // Raridade (5 = lendário)
                null, // Tipo primário
                null, // Tipo secundário
                null, // Doce
                0  // Distância
            );
            
            return chest;
        }
        
        // Gera um índice aleatório para Pokémon normal
        int indiceAleatorio = (int) (Math.random() * pokemons.size());
        
        // Retorna o Pokémon no índice sorteado
        return pokemons.get(indiceAleatorio);
    }
}