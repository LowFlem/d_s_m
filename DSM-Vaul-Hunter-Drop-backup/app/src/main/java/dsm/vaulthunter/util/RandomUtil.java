package dsm.vaulthunter.util;

import android.location.Location;

import java.util.ArrayList;
import java.util.List;
import java.util.Random;

import org.osmdroid.util.GeoPoint;

/**
 * Utility class for random generation functions
 */
public class RandomUtil {
    
    private static final Random random = new Random();
    
    /**
     * Generate a random point within a radius around a center point
     * @param center Center point
     * @param radiusMeters Radius in meters
     * @return Random GeoPoint within the radius
     */
    public static GeoPoint randomPointAround(GeoPoint center, double radiusMeters) {
        // Convert radius from meters to degrees
        double radiusInDegrees = radiusMeters / 111320.0; // 1 degree is approximately 111320 meters
        
        // Generate random point using uniform distribution
        double u = random.nextDouble();
        double v = random.nextDouble();
        double w = radiusInDegrees * Math.sqrt(u);
        double t = 2 * Math.PI * v;
        double x = w * Math.cos(t);
        double y = w * Math.sin(t);
        
        // Adjust for latitude's distortion of longitude distance
        double newX = x / Math.cos(Math.toRadians(center.getLatitude()));
        
        // Calculate new point
        double newLat = center.getLatitude() + y;
        double newLon = center.getLongitude() + newX;
        
        return new GeoPoint(newLat, newLon);
    }
    
    /**
     * Generate multiple random points within a radius around a center point
     * @param center Center point
     * @param radiusMeters Radius in meters
     * @param count Number of points to generate
     * @return List of random GeoPoints
     */
    public static List<GeoPoint> randomPointsAround(GeoPoint center, double radiusMeters, int count) {
        List<GeoPoint> points = new ArrayList<>(count);
        
        for (int i = 0; i < count; i++) {
            points.add(randomPointAround(center, radiusMeters));
        }
        
        return points;
    }
    
    /**
     * Generate a weighted random index based on the provided weights
     * @param weights Array of weights
     * @return Random index
     */
    public static int weightedRandomIndex(int[] weights) {
        int totalWeight = 0;
        
        for (int weight : weights) {
            totalWeight += weight;
        }
        
        if (totalWeight <= 0) {
            return 0;
        }
        
        int randomValue = random.nextInt(totalWeight);
        int currentWeight = 0;
        
        for (int i = 0; i < weights.length; i++) {
            currentWeight += weights[i];
            if (randomValue < currentWeight) {
                return i;
            }
        }
        
        return weights.length - 1;
    }
    
    /**
     * Generate a random number in a given range
     * @param min Minimum value (inclusive)
     * @param max Maximum value (inclusive)
     * @return Random number in the range
     */
    public static int randomInRange(int min, int max) {
        return random.nextInt((max - min) + 1) + min;
    }
    
    /**
     * Generate a random float in a given range
     * @param min Minimum value (inclusive)
     * @param max Maximum value (inclusive)
     * @return Random float in the range
     */
    public static float randomInRange(float min, float max) {
        return min + random.nextFloat() * (max - min);
    }
    
    /**
     * Generate a random double in a given range
     * @param min Minimum value (inclusive)
     * @param max Maximum value (inclusive)
     * @return Random double in the range
     */
    public static double randomInRange(double min, double max) {
        return min + random.nextDouble() * (max - min);
    }
    
    /**
     * Calculate distance between two points in meters
     * @param lat1 Latitude of point 1
     * @param lon1 Longitude of point 1
     * @param lat2 Latitude of point 2
     * @param lon2 Longitude of point 2
     * @return Distance in meters
     */
    public static float distanceBetween(double lat1, double lon1, double lat2, double lon2) {
        Location location1 = new Location("point1");
        location1.setLatitude(lat1);
        location1.setLongitude(lon1);
        
        Location location2 = new Location("point2");
        location2.setLatitude(lat2);
        location2.setLongitude(lon2);
        
        return location1.distanceTo(location2);
    }
    
    /**
     * Get a random element from a list
     * @param list List to get random element from
     * @param <T> Type of elements in the list
     * @return Random element from the list, or null if the list is empty
     */
    public static <T> T randomElement(List<T> list) {
        if (list == null || list.isEmpty()) {
            return null;
        }
        
        return list.get(random.nextInt(list.size()));
    }
    
    /**
     * Generate a random boolean with the given probability of being true
     * @param probability Probability of returning true (0.0 to 1.0)
     * @return Random boolean
     */
    public static boolean randomBoolean(double probability) {
        return random.nextDouble() < probability;
    }
    
    /**
     * Shuffle a list in place
     * @param list List to shuffle
     * @param <T> Type of elements in the list
     */
    public static <T> void shuffle(List<T> list) {
        int n = list.size();
        
        for (int i = n - 1; i > 0; i--) {
            int j = random.nextInt(i + 1);
            T temp = list.get(i);
            list.set(i, list.get(j));
            list.set(j, temp);
        }
    }
}