import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:flutter_test_gui/consts/consts.dart';

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
      backgroundColor: Constants.backColor,
      appBar: AppBar(
        title: const Text('Settings'),
        backgroundColor: const Color(0xFF292c3c), //0xFF282C34),
        foregroundColor: Constants.textColor,
      ),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          children: [
            TextField(
              controller: _appEnvironmentController,
              decoration: const InputDecoration(
                  labelText: 'App Environment',
                  labelStyle: TextStyle(color: Constants.highlightColor)),
              style: const TextStyle(color: Constants.textColor),
            ),
            TextField(
              controller: _appHostController,
              decoration: const InputDecoration(
                  labelText: 'App Host',
                  labelStyle: TextStyle(color: Constants.highlightColor)),
              style: const TextStyle(color: Constants.textColor),
            ),
            TextField(
              controller: _appPortController,
              decoration: const InputDecoration(
                  labelText: 'App Port',
                  labelStyle: TextStyle(color: Constants.highlightColor)),
              style: const TextStyle(color: Constants.textColor),
            ),
            TextField(
              controller: _appOriginController,
              decoration: const InputDecoration(
                  labelText: 'App Origin',
                  labelStyle: TextStyle(color: Constants.highlightColor)),
              style: const TextStyle(color: Constants.textColor),
            ),
            TextField(
              controller: _appRelayController,
              decoration: const InputDecoration(
                  labelText: 'App Relay',
                  labelStyle: TextStyle(color: Constants.highlightColor)),
              style: const TextStyle(color: Constants.textColor),
            ),
            Spacer(),
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
