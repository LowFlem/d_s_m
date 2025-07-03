package dsm.vaulthunter.util;

import java.text.SimpleDateFormat;
import java.util.Calendar;
import java.util.Date;
import java.util.Locale;
import java.util.TimeZone;
import java.util.concurrent.TimeUnit;

/**
 * Utility class for time and date operations
 */
public class TimeUtil {
    
    // Standard date format patterns
    private static final String ISO_DATE_FORMAT = "yyyy-MM-dd'T'HH:mm:ss'Z'";
    private static final String DATE_ONLY_FORMAT = "yyyy-MM-dd";
    private static final String TIME_ONLY_FORMAT = "HH:mm:ss";
    private static final String FRIENDLY_DATE_FORMAT = "MMM dd, yyyy";
    private static final String FRIENDLY_DATETIME_FORMAT = "MMM dd, yyyy hh:mm a";
    
    /**
     * Format a timestamp to ISO-8601 format
     * @param timestamp Timestamp in milliseconds
     * @return Formatted date string
     */
    public static String formatIsoDate(long timestamp) {
        SimpleDateFormat sdf = new SimpleDateFormat(ISO_DATE_FORMAT, Locale.US);
        sdf.setTimeZone(TimeZone.getTimeZone("UTC"));
        return sdf.format(new Date(timestamp));
    }
    
    /**
     * Format a timestamp to date-only format (yyyy-MM-dd)
     * @param timestamp Timestamp in milliseconds
     * @return Formatted date string
     */
    public static String formatDateOnly(long timestamp) {
        SimpleDateFormat sdf = new SimpleDateFormat(DATE_ONLY_FORMAT, Locale.US);
        return sdf.format(new Date(timestamp));
    }
    
    /**
     * Format a timestamp to time-only format (HH:mm:ss)
     * @param timestamp Timestamp in milliseconds
     * @return Formatted time string
     */
    public static String formatTimeOnly(long timestamp) {
        SimpleDateFormat sdf = new SimpleDateFormat(TIME_ONLY_FORMAT, Locale.US);
        return sdf.format(new Date(timestamp));
    }
    
    /**
     * Format a timestamp to friendly date format (MMM dd, yyyy)
     * @param timestamp Timestamp in milliseconds
     * @return Formatted date string
     */
    public static String formatFriendlyDate(long timestamp) {
        SimpleDateFormat sdf = new SimpleDateFormat(FRIENDLY_DATE_FORMAT, Locale.US);
        return sdf.format(new Date(timestamp));
    }
    
    /**
     * Format a timestamp to friendly date and time format (MMM dd, yyyy hh:mm a)
     * @param timestamp Timestamp in milliseconds
     * @return Formatted date and time string
     */
    public static String formatFriendlyDateTime(long timestamp) {
        SimpleDateFormat sdf = new SimpleDateFormat(FRIENDLY_DATETIME_FORMAT, Locale.US);
        return sdf.format(new Date(timestamp));
    }
    
    /**
     * Get a human-readable time ago string (e.g., "2 hours ago", "5 days ago")
     * @param timestamp Timestamp in milliseconds
     * @return Time ago string
     */
    public static String getTimeAgo(long timestamp) {
        long now = System.currentTimeMillis();
        long diff = now - timestamp;
        
        // Convert to seconds
        long seconds = TimeUnit.MILLISECONDS.toSeconds(diff);
        
        if (seconds < 60) {
            return "just now";
        }
        
        // Convert to minutes
        long minutes = TimeUnit.SECONDS.toMinutes(seconds);
        
        if (minutes < 60) {
            return minutes + (minutes == 1 ? " minute ago" : " minutes ago");
        }
        
        // Convert to hours
        long hours = TimeUnit.MINUTES.toHours(minutes);
        
        if (hours < 24) {
            return hours + (hours == 1 ? " hour ago" : " hours ago");
        }
        
        // Convert to days
        long days = TimeUnit.HOURS.toDays(hours);
        
        if (days < 7) {
            return days + (days == 1 ? " day ago" : " days ago");
        }
        
        // Convert to weeks
        long weeks = days / 7;
        
        if (weeks < 4) {
            return weeks + (weeks == 1 ? " week ago" : " weeks ago");
        }
        
        // Convert to months
        long months = days / 30;
        
        if (months < 12) {
            return months + (months == 1 ? " month ago" : " months ago");
        }
        
        // Convert to years
        long years = days / 365;
        
        return years + (years == 1 ? " year ago" : " years ago");
    }
    
    /**
     * Get the start of day timestamp for a given timestamp
     * @param timestamp Timestamp in milliseconds
     * @return Start of day timestamp
     */
    public static long getStartOfDay(long timestamp) {
        Calendar calendar = Calendar.getInstance();
        calendar.setTimeInMillis(timestamp);
        calendar.set(Calendar.HOUR_OF_DAY, 0);
        calendar.set(Calendar.MINUTE, 0);
        calendar.set(Calendar.SECOND, 0);
        calendar.set(Calendar.MILLISECOND, 0);
        
        return calendar.getTimeInMillis();
    }
    
    /**
     * Get the end of day timestamp for a given timestamp
     * @param timestamp Timestamp in milliseconds
     * @return End of day timestamp
     */
    public static long getEndOfDay(long timestamp) {
        Calendar calendar = Calendar.getInstance();
        calendar.setTimeInMillis(timestamp);
        calendar.set(Calendar.HOUR_OF_DAY, 23);
        calendar.set(Calendar.MINUTE, 59);
        calendar.set(Calendar.SECOND, 59);
        calendar.set(Calendar.MILLISECOND, 999);
        
        return calendar.getTimeInMillis();
    }
    
    /**
     * Get the start of week (Sunday) timestamp for a given timestamp
     * @param timestamp Timestamp in milliseconds
     * @return Start of week timestamp
     */
    public static long getStartOfWeek(long timestamp) {
        Calendar calendar = Calendar.getInstance();
        calendar.setTimeInMillis(timestamp);
        calendar.set(Calendar.DAY_OF_WEEK, Calendar.SUNDAY);
        calendar.set(Calendar.HOUR_OF_DAY, 0);
        calendar.set(Calendar.MINUTE, 0);
        calendar.set(Calendar.SECOND, 0);
        calendar.set(Calendar.MILLISECOND, 0);
        
        return calendar.getTimeInMillis();
    }
    
    /**
     * Check if two timestamps are on the same day
     * @param timestamp1 First timestamp
     * @param timestamp2 Second timestamp
     * @return true if both timestamps are on the same day
     */
    public static boolean isSameDay(long timestamp1, long timestamp2) {
        Calendar cal1 = Calendar.getInstance();
        Calendar cal2 = Calendar.getInstance();
        cal1.setTimeInMillis(timestamp1);
        cal2.setTimeInMillis(timestamp2);
        
        return cal1.get(Calendar.YEAR) == cal2.get(Calendar.YEAR) &&
               cal1.get(Calendar.DAY_OF_YEAR) == cal2.get(Calendar.DAY_OF_YEAR);
    }
    
    /**
     * Get the current timestamp in milliseconds
     * @return Current timestamp
     */
    public static long getCurrentTime() {
        return System.currentTimeMillis();
    }
}