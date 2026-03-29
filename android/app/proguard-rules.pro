# Unlicense — cochranblock.org
# Keep JNI native methods — proguard must not strip these
-keep class org.cochranblock.pocketserver.PocketServer {
    native <methods>;
}
