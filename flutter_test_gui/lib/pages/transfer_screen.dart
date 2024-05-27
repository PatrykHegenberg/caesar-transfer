import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test_gui/main.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:cross_file/cross_file.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:flutter_test_gui/src/rust/api/simple.dart';
// import 'package:flutter_test_gui/src/rust/frb_generated.dart';
import 'package:flutter_test_gui/consts/consts.dart';
import 'package:permission_handler/permission_handler.dart';

class TransferScreen extends StatefulWidget {
  final String transferName;
  final String directory;

  const TransferScreen(
      {Key? key, required this.transferName, required this.directory})
      : super(key: key);

  @override
  TransferScreenState createState() => TransferScreenState();
}

class TransferScreenState extends State<TransferScreen> {
  String appOrigin = '';
  String inputValue = '';
  @override
  void initState() {
    super.initState();
    loadSettings().then((_) => callStartReceiver(appOrigin));
  }

  Future<void> loadSettings() async {
    SharedPreferences prefs = await SharedPreferences.getInstance();
    appOrigin = prefs.getString('app_origin') ??
        'wss://caesar-transfer-iu.shuttleapp.rs';
  }

  Future<void> callStartReceiver(String appOrigin) async {
    _startTransfer(appOrigin);
  }

  Future<void> _startTransfer(String appOrigin) async {
    final input = widget.transferName;
    String filePath = widget.directory;
    if (input.isNotEmpty) {
      if (Platform.isAndroid) {
        if (await _requestPermission(Permission.manageExternalStorage)) {
          try {
            final outcome = await startRustReceiver(
                filepath: filePath, transfername: input, relay: appOrigin);
            print('Ergebnis von Rust: $outcome');
          } catch (e) {
            print('Fehler beim Starten des Receivers: $e');
          }
          Navigator.push(
              context,
              MaterialPageRoute(
                  builder: (context) =>
                      const MyHomePage(title: 'Caesar Transfer')));
        } else {
          Navigator.push(
              context,
              MaterialPageRoute(
                  builder: (context) =>
                      const MyHomePage(title: 'Caesar Transfer')));
        }
      } else {
        try {
          final outcome = await startRustReceiver(
              filepath: filePath, transfername: input, relay: appOrigin);
          print('Ergebnis von Rust: $outcome');
        } catch (e) {
          print('Fehler beim Starten des Receivers: $e');
        }
        Navigator.push(
            context,
            MaterialPageRoute(
                builder: (context) =>
                    const MyHomePage(title: 'Caesar Transfer')));
      }
    }
    print("Transfer startet with app_origin: $appOrigin");
  }

  Future<bool> _requestPermission(Permission permission) async {
    if (await permission.isGranted) {
      return true;
    } else {
      var result = await permission.request();
      if (result == PermissionStatus.granted) {
        return true;
      } else {
        return false;
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Constants.backColor,
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text(
              widget.transferName,
              style: const TextStyle(color: Colors.white, fontSize: 24),
            ),
            Text("Transfer in Progress"),
            const SizedBox(height: 32),
            const Center(
              child: Icon(
                Icons.cloud_download_rounded,
                color: Constants.highlightColor,
                size: 200,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
