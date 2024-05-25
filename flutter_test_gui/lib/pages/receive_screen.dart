import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test_gui/main.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:permission_handler/permission_handler.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:flutter_test_gui/src/rust/api/simple.dart';
// import 'package:flutter_test_gui/src/rust/frb_generated.dart';
import 'package:flutter_test_gui/consts/consts.dart';

class ReceiveScreen extends StatefulWidget {
  const ReceiveScreen({super.key});

  @override
  ReceiveScreenState createState() => ReceiveScreenState();
}

class ReceiveScreenState extends State<ReceiveScreen> {
  String appOrigin = '';
  final myController = TextEditingController();
  String inputValue = '';
  bool _showScanner = false;

  Widget _buildQRScanner() {
    if (Platform.isIOS || Platform.isAndroid) {
      return MobileScanner(
        controller: MobileScannerController(
            detectionSpeed: DetectionSpeed.noDuplicates),
        onDetect: (barcode) {
          if (barcode.raw == null) {
            debugPrint('Failed to scan qr code');
          } else {
            final String code = barcode.raw.toString();
            setState(() {
              inputValue = code;
              _showScanner = false;
            });
          }
        },
      );
    } else {
      _showScanner = false;
      return const SizedBox.shrink();
    }
  }

  Future<void> loadSettings() async {
    SharedPreferences prefs = await SharedPreferences.getInstance();
    appOrigin = prefs.getString('app_origin') ??
        'wss://caesar-transfer-iu.shuttleapp.rs'; // Laden Sie die app_origin
  }

  Future<bool> _requestPermission(Permission permission) async {
    print("In _requestPermission");
    if (await permission.isGranted) {
      print("Granted");
      return true;
    } else {
      print("Else Zweig");
      final result = await permission.request();
      if (result == PermissionStatus.granted) {
        return true;
      } else {
        return false;
      }
    }
  }

  Future<void> _startTransfer(String appOrigin) async {
    final input = inputValue.trim();
    if (input.isNotEmpty) {
      if (Platform.isAndroid) {
        if (await _requestPermission(Permission.manageExternalStorage)) {
          try {
            final outcome =
                await startRustReceiver(transfername: input, relay: appOrigin);
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
          final outcome =
              await startRustReceiver(transfername: input, relay: appOrigin);
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

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: backColor,
      body: Center(
          child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          if (!_showScanner)
            GestureDetector(
              onTap: () {
                if (Platform.isIOS || Platform.isAndroid) {
                  setState(() {
                    _showScanner = true;
                  });
                }
              },
              child: Container(
                width: 200,
                height: 200,
                decoration: const BoxDecoration(
                  shape: BoxShape.circle,
                  color: Constants.textColor,
                ),
                child: const Center(
                  child: Icon(
                    Icons.qr_code,
                    color: Constants.highlightColor,
                    size: 100,
                  ),
                ),
              ),
            ),
          if (_showScanner)
            Container(
              width: MediaQuery.of(context).size.width * 0.8,
              height: MediaQuery.of(context).size.height * 0.5,
              decoration: BoxDecoration(
                color: Colors.white,
                borderRadius: BorderRadius.circular(16),
              ),
              child: _buildQRScanner(),
            ),
          const SizedBox(height: 32),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: SizedBox(
              width: MediaQuery.of(context).size.width * 0.5,
              child: TextField(
                controller: myController,
                textAlign: TextAlign.center,
                style: const TextStyle(
                  color: Constants.highlightColor,
                ),
                onChanged: (value) {
                  setState(() {
                    inputValue = value;
                  });
                },
                decoration: const InputDecoration(
                  labelText: 'Enter Transfername',
                  alignLabelWithHint: true,
                  floatingLabelAlignment: FloatingLabelAlignment.center,
                  labelStyle: TextStyle(color: Constants.textColor),
                  enabledBorder: UnderlineInputBorder(
                    borderSide: BorderSide(color: Constants.textColor),
                  ),
                  focusedBorder: UnderlineInputBorder(
                    borderSide: BorderSide(color: Constants.textColor),
                  ),
                ),
              ),
            ),
          ),
          const SizedBox(height: 16),
          ElevatedButton(
            style: ElevatedButton.styleFrom(
              backgroundColor: Constants.textColor,
              foregroundColor: Constants.highlightColor,
              shape: RoundedRectangleBorder(
                borderRadius: BorderRadius.circular(20),
              ),
            ),
            onPressed: () {
              loadSettings().then((_) => _startTransfer(appOrigin));
            },
            child: const Text('Receive'),
          ),
        ],
      )),
    );
  }
}
