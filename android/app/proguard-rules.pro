# Proguard rules for Whirled Peas Visualiser
# The app is primarily native code, so we need minimal Java rules

# Keep the main activity
-keep class commercial_and_residential_evil.whirled_peas.MainActivity { *; }

# Keep GameActivity
-keep class com.google.androidgamesdk.GameActivity { *; }

# Keep native methods
-keepclasseswithmembernames class * {
    native <methods>;
}

# Don't warn about missing classes from games SDK
-dontwarn com.google.androidgamesdk.**
