import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:flutter_test_gui/consts/consts.dart';

/// Screen for displaying and editing the app's settings.
///
/// This screen allows the user to view and edit the app's settings.
/// The settings include the app environment, host, port, origin, and relay.
/// The settings are stored in SharedPreferences.
class SettingsScreen extends StatefulWidget {
  /// Constructs a [SettingsScreen].
  const SettingsScreen({Key? key}) : super(key: key);

  @override
  _SettingsScreenState createState() => _SettingsScreenState();
}

/// State for the [SettingsScreen].
///
/// This state manages the text editing controllers for the app's settings.
/// It also loads the settings from SharedPreferences when the widget is
/// first created.
class _SettingsScreenState extends State<SettingsScreen> {
  // Controllers for the text fields.
  final TextEditingController _appEnvironmentController =
      TextEditingController();
  final TextEditingController _appHostController = TextEditingController();
  final TextEditingController _appPortController = TextEditingController();
  final TextEditingController _appOriginController = TextEditingController();
  final TextEditingController _appRelayController = TextEditingController();

  /// Loads the app settings from SharedPreferences when the widget is created.
  @override
  void initState() {
    super.initState();
    loadSettings();
  }

  /// Loads the app settings from SharedPreferences.
  ///
  /// This function retrieves the app settings from SharedPreferences and
  /// sets the text of the corresponding text editing controllers.
  /// If any of the settings are not found in SharedPreferences, an empty
  /// string is used instead.
  Future<void> loadSettings() async {
    // Retrieve the SharedPreferences instance
    SharedPreferences prefs = await SharedPreferences.getInstance();

    // Retrieve the app settings from SharedPreferences
    String appEnvironment = prefs.getString('app_environment') ?? '';
    String appHost = prefs.getString('app_host') ?? '';
    String appPort = prefs.getString('app_port') ?? '';
    String appOrigin = prefs.getString('app_origin') ?? '';
    String appRelay = prefs.getString('app_relay') ?? '';

    // Set the text of the corresponding text editing controllers
    setState(() {
      _appEnvironmentController.text = appEnvironment;
      _appHostController.text = appHost;
      _appPortController.text = appPort;
      _appOriginController.text = appOrigin;
      _appRelayController.text = appRelay;
    });
  }

  /// Saves the app settings to SharedPreferences.
  ///
  /// This function retrieves the text from the corresponding text editing controllers
  /// and saves them to SharedPreferences.
  /// If any of the settings are empty, it saves an empty string.
  Future<void> saveSettings() async {
    // Retrieve the SharedPreferences instance
    SharedPreferences prefs = await SharedPreferences.getInstance();

    // Retrieve the text from the corresponding text editing controllers
    String appEnvironment = _appEnvironmentController.text;
    String appHost = _appHostController.text;
    String appPort = _appPortController.text;
    String appOrigin = _appOriginController.text;
    String appRelay = _appRelayController.text;

    // Save the app settings to SharedPreferences
    await prefs.setString('app_environment', appEnvironment);
    await prefs.setString('app_host', appHost);
    await prefs.setString('app_port', appPort);
    await prefs.setString('app_origin', appOrigin);
    await prefs.setString('app_relay', appRelay);
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      // Set the background color of the Scaffold
      backgroundColor: Constants.backColor,
      appBar: AppBar(
        // Set the title of the AppBar
        title: const Text('Settings'),
        // Set the background color of the AppBar
        backgroundColor: const Color(0xFF292c3c), //0xFF282C34),
        // Set the foreground color of the AppBar
        foregroundColor: Constants.textColor,
      ),
      body: Padding(
        // Set the padding around the body of the Scaffold
        padding: const EdgeInsets.all(16.0),
        child: Column(
          children: [
            // Create a TextField for the 'App Environment' setting
            TextField(
              controller: _appEnvironmentController,
              decoration: const InputDecoration(
                  labelText: 'App Environment',
                  labelStyle: TextStyle(color: Constants.highlightColor)),
              style: const TextStyle(color: Constants.textColor),
            ),
            // Create a TextField for the 'App Host' setting
            TextField(
              controller: _appHostController,
              decoration: const InputDecoration(
                  labelText: 'App Host',
                  labelStyle: TextStyle(color: Constants.highlightColor)),
              style: const TextStyle(color: Constants.textColor),
            ),
            // Create a TextField for the 'App Port' setting
            TextField(
              controller: _appPortController,
              decoration: const InputDecoration(
                  labelText: 'App Port',
                  labelStyle: TextStyle(color: Constants.highlightColor)),
              style: const TextStyle(color: Constants.textColor),
            ),
            // Create a TextField for the 'App Origin' setting
            TextField(
              controller: _appOriginController,
              decoration: const InputDecoration(
                  labelText: 'App Origin',
                  labelStyle: TextStyle(color: Constants.highlightColor)),
              style: const TextStyle(color: Constants.textColor),
            ),
            // Create a TextField for the 'App Relay' setting
            TextField(
              controller: _appRelayController,
              decoration: const InputDecoration(
                  labelText: 'App Relay',
                  labelStyle: TextStyle(color: Constants.highlightColor)),
              style: const TextStyle(color: Constants.textColor),
            ),
            Spacer(),
            // Create an ElevatedButton to save the settings and return to the previous screen
            ElevatedButton(
              style: ElevatedButton.styleFrom(
                backgroundColor: Constants.textColor,
                foregroundColor: Constants.backColor,
                shape: RoundedRectangleBorder(
                  borderRadius: BorderRadius.circular(20),
                ),
              ),
              onPressed: () async {
                await saveSettings();
                Navigator.pop(context);
              },
              child: const Text('Save'),
            ),
          ],
        ),
      ),
    );
  }
}
