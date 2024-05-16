import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';

class SettingsScreen extends StatefulWidget {
  @override
  _SettingsScreenState createState() => _SettingsScreenState();
}

class _SettingsScreenState extends State<SettingsScreen> {
  final TextEditingController _appEnvironmentController =
      TextEditingController();
  final TextEditingController _appHostController = TextEditingController();
  final TextEditingController _appPortController = TextEditingController();
  final TextEditingController _appOriginController = TextEditingController();
  final TextEditingController _appRelayController = TextEditingController();

  @override
  void initState() {
    super.initState();
    loadSettings();
  }

  Future<void> loadSettings() async {
    SharedPreferences prefs = await SharedPreferences.getInstance();
    String appEnvironment = prefs.getString('app_environment') ?? '';
    String appHost = prefs.getString('app_host') ?? '';
    String appPort = prefs.getString('app_port') ?? '';
    String appOrigin = prefs.getString('app_origin') ?? '';
    String appRelay = prefs.getString('app_relay') ?? '';

    // Setzen Sie die Controller-Texte nach dem Laden der Einstellungen
    setState(() {
      _appEnvironmentController.text = appEnvironment;
      _appHostController.text = appHost;
      _appPortController.text = appPort;
      _appOriginController.text = appOrigin;
      _appRelayController.text = appRelay;
    });
  }
  // Future<void> loadSettings() async {
  //   SharedPreferences prefs = await SharedPreferences.getInstance();
  //   setState(() {
  //     _appEnvironmentController.text = prefs.getString('app_environment') ?? '';
  //     _appHostController.text = prefs.getString('app_host') ?? '';
  //     _appPortController.text = prefs.getString('app_port') ?? '';
  //     _appOriginController.text = prefs.getString('app_origin') ?? '';
  //     _appRelayController.text = prefs.getString('app_relay') ?? '';
  //   });
  // }

  Future<void> saveSettings() async {
    SharedPreferences prefs = await SharedPreferences.getInstance();
    await prefs.setString('app_environment', _appEnvironmentController.text);
    await prefs.setString('app_host', _appHostController.text);
    await prefs.setString('app_port', _appPortController.text);
    await prefs.setString('app_origin', _appOriginController.text);
    await prefs.setString('app_relay', _appRelayController.text);
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('Settings'),
      ),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          children: [
            TextField(
              controller: _appEnvironmentController,
              decoration: InputDecoration(labelText: 'App Environment'),
            ),
            TextField(
              controller: _appHostController,
              decoration: InputDecoration(labelText: 'App Host'),
            ),
            TextField(
              controller: _appPortController,
              decoration: InputDecoration(labelText: 'App Port'),
            ),
            TextField(
              controller: _appOriginController,
              decoration: InputDecoration(labelText: 'App Origin'),
            ),
            TextField(
              controller: _appRelayController,
              decoration: InputDecoration(labelText: 'App Relay'),
            ),
            ElevatedButton(
              onPressed: () async {
                await saveSettings();
                Navigator.pop(context);
              },
              child: Text('Save'),
            ),
          ],
        ),
      ),
    );
  }
}
